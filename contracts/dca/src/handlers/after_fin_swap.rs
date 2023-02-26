use std::cmp::min;

use crate::contract::{AFTER_BANK_SWAP_REPLY_ID, AFTER_Z_DELEGATION_REPLY_ID};
use crate::error::ContractError;
use crate::helpers::vault_helpers::{get_swap_amount, has_sufficient_funds};
use crate::state::cache::{CACHE, SWAP_CACHE};
use crate::state::config::{get_config, get_custom_fee};
use crate::state::events::create_event;
use crate::state::triggers::{delete_trigger, save_trigger};
use crate::state::vaults::{get_vault, update_vault};

use base::events::event::{EventBuilder, EventData, ExecutionSkippedReason};
use base::helpers::coin_helpers::add_to_coin;
use base::helpers::community_pool::create_fund_community_pool_msg;
use base::helpers::math_helpers::checked_mul;
use base::helpers::time_helpers::get_next_target_time;
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::{PostExecutionAction, VaultStatus};
use cosmwasm_std::{to_binary, SubMsg, SubMsgResult, WasmMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Attribute, BankMsg, Coin, CosmosMsg, DepsMut, Env, Reply, Response, Uint128};
use staking_router::msg::ExecuteMsg as StakingRouterExecuteMsg;

pub fn after_fin_swap(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let mut vault = get_vault(deps.storage, cache.vault_id.into())?;

    let mut attributes: Vec<Attribute> = Vec::new();
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut sub_msgs: Vec<SubMsg> = Vec::new();

    delete_trigger(deps.storage, vault.id)?;

    match reply.result {
        SubMsgResult::Ok(_) => {
            let swap_cache = SWAP_CACHE.load(deps.storage)?;

            let swap_denom_balance = &deps
                .querier
                .query_balance(&env.contract.address, &vault.get_swap_denom())?;

            let receive_denom_balance = &deps
                .querier
                .query_balance(&env.contract.address, &vault.get_receive_denom())?;

            let coin_sent = Coin::new(
                (swap_cache.swap_denom_balance.amount - swap_denom_balance.amount).into(),
                swap_denom_balance.denom.clone(),
            );

            let coin_received = Coin::new(
                (receive_denom_balance.amount - swap_cache.receive_denom_balance.amount).into(),
                receive_denom_balance.denom.clone(),
            );

            let config = get_config(deps.storage)?;

            let fee_percent = match (
                get_custom_fee(deps.storage, vault.get_swap_denom()),
                get_custom_fee(deps.storage, vault.get_receive_denom()),
            ) {
                (Some(swap_denom_fee_percent), Some(receive_denom_fee_percent)) => {
                    min(swap_denom_fee_percent, receive_denom_fee_percent)
                }
                (Some(swap_denom_fee_percent), None) => swap_denom_fee_percent,
                (None, Some(receive_denom_fee_percent)) => receive_denom_fee_percent,
                (None, None) => config.swap_fee_percent,
            };

            let automation_fee_rate = config.delegation_fee_percent.checked_mul(
                vault
                    .destinations
                    .iter()
                    .filter(|destination| destination.action == PostExecutionAction::ZDelegate)
                    .map(|destination| destination.allocation)
                    .sum(),
            )?;

            let swap_fee = checked_mul(coin_received.amount, fee_percent)?;
            let total_after_swap_fee = coin_received.amount - swap_fee;
            let automation_fee = checked_mul(total_after_swap_fee, automation_fee_rate)?;

            config.fee_collectors.iter().for_each(|fee_collector| {
                let swap_fee_allocation = Coin::new(
                    checked_mul(swap_fee, fee_collector.allocation)
                        .ok()
                        .expect("amount to be distributed should be valid")
                        .into(),
                    coin_received.denom.clone(),
                );

                if swap_fee_allocation.amount.gt(&Uint128::zero()) {
                    match fee_collector.address.as_str() {
                        "community_pool" => messages.push(create_fund_community_pool_msg(
                            env.contract.address.to_string(),
                            vec![swap_fee_allocation.clone()],
                        )),
                        _ => {
                            messages.push(CosmosMsg::Bank(BankMsg::Send {
                                to_address: fee_collector.address.to_string(),
                                amount: vec![swap_fee_allocation],
                            }));
                        }
                    }
                }

                let automation_fee_allocation = Coin::new(
                    checked_mul(automation_fee, fee_collector.allocation)
                        .ok()
                        .expect("amount to be distributed should be valid")
                        .into(),
                    coin_received.denom.clone(),
                );

                if automation_fee_allocation.amount.gt(&Uint128::zero()) {
                    match fee_collector.address.as_str() {
                        "community_pool" => messages.push(create_fund_community_pool_msg(
                            env.contract.address.to_string(),
                            vec![automation_fee_allocation.clone()],
                        )),
                        _ => {
                            messages.push(CosmosMsg::Bank(BankMsg::Send {
                                to_address: fee_collector.address.to_string(),
                                amount: vec![automation_fee_allocation],
                            }));
                        }
                    }
                }
            });

            let total_fee = swap_fee + automation_fee;
            let total_after_total_fee = coin_received.amount - total_fee;

            vault.destinations.iter().for_each(|destination| {
                let allocation_amount = Coin::new(
                    checked_mul(total_after_total_fee, destination.allocation)
                        .ok()
                        .expect("amount to be distributed should be valid")
                        .into(),
                    coin_received.denom.clone(),
                );

                if allocation_amount.amount.gt(&Uint128::zero()) {
                    match destination.action {
                        PostExecutionAction::Send => {
                            messages.push(CosmosMsg::Bank(BankMsg::Send {
                                to_address: destination.address.to_string(),
                                amount: vec![allocation_amount],
                            }))
                        }
                        PostExecutionAction::ZDelegate => {
                            sub_msgs.push(SubMsg::reply_on_success(
                                BankMsg::Send {
                                    to_address: vault.owner.to_string(),
                                    amount: vec![allocation_amount.clone()],
                                },
                                AFTER_BANK_SWAP_REPLY_ID,
                            ));
                            sub_msgs.push(SubMsg::reply_always(
                                CosmosMsg::Wasm(WasmMsg::Execute {
                                    contract_addr: config.staking_router_address.to_string(),
                                    msg: to_binary(&StakingRouterExecuteMsg::ZDelegate {
                                        delegator_address: vault.owner.clone(),
                                        validator_address: destination.address.clone(),
                                        denom: allocation_amount.denom.clone(),
                                        amount: allocation_amount.amount.clone(),
                                    })
                                    .unwrap(),
                                    funds: vec![],
                                }),
                                AFTER_Z_DELEGATION_REPLY_ID,
                            ));
                        }
                    }
                }
            });

            vault.balance.amount -= get_swap_amount(vault.clone(), &deps.as_ref())?.amount;

            if !has_sufficient_funds(vault.clone(), &deps.as_ref())? {
                vault.status = VaultStatus::Inactive;
            }

            vault.swapped_amount = add_to_coin(vault.swapped_amount, coin_sent.amount)?;
            vault.received_amount = add_to_coin(vault.received_amount, total_after_total_fee)?;

            update_vault(deps.storage, &vault)?;

            if vault.is_active() {
                save_trigger(
                    deps.storage,
                    Trigger {
                        vault_id: vault.id,
                        configuration: TriggerConfiguration::Time {
                            target_time: get_next_target_time(
                                env.block.time,
                                match vault.trigger {
                                    Some(TriggerConfiguration::Time { target_time }) => target_time,
                                    _ => env.block.time,
                                },
                                vault.time_interval,
                            ),
                        },
                    },
                )?;
            }

            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block,
                    EventData::DcaVaultExecutionCompleted {
                        sent: coin_sent.clone(),
                        received: coin_received.clone(),
                        fee: Coin::new(total_fee.into(), coin_received.denom),
                    },
                ),
            )?;

            attributes.push(Attribute::new("status", "success"));
        }
        SubMsgResult::Err(_) => {
            if !has_sufficient_funds(vault.clone(), &deps.as_ref())? {
                create_event(
                    deps.storage,
                    EventBuilder::new(
                        vault.id,
                        env.block.to_owned(),
                        EventData::DcaVaultExecutionSkipped {
                            reason: ExecutionSkippedReason::UnknownFailure,
                        },
                    ),
                )?;

                vault.status = VaultStatus::Inactive;
                update_vault(deps.storage, &vault)?;
            } else {
                create_event(
                    deps.storage,
                    EventBuilder::new(
                        vault.id,
                        env.block.to_owned(),
                        EventData::DcaVaultExecutionSkipped {
                            reason: ExecutionSkippedReason::SlippageToleranceExceeded,
                        },
                    ),
                )?;

                save_trigger(
                    deps.storage,
                    Trigger {
                        vault_id: vault.id,
                        configuration: TriggerConfiguration::Time {
                            target_time: get_next_target_time(
                                env.block.time,
                                match vault.trigger.expect("msg") {
                                    TriggerConfiguration::Time { target_time } => target_time,
                                    _ => env.block.time,
                                },
                                vault.time_interval,
                            ),
                        },
                    },
                )?;
            }

            attributes.push(Attribute::new("status", "skipped"));
        }
    }

    Ok(Response::new()
        .add_attribute("method", "fin_swap_completed")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id)
        .add_attributes(attributes)
        .add_messages(messages)
        .add_submessages(sub_msgs))
}
