use crate::constants::ONE_HUNDRED;
use crate::contract::DELEGATION_SUCCEEDED_ID;
use crate::error::ContractError;
use crate::state::{
    create_event, remove_trigger, save_trigger, vault_store, CACHE, CONFIG, LIMIT_ORDER_CACHE,
};
use crate::vault::Vault;
use base::events::event::{EventBuilder, EventData};
use base::helpers::time_helpers::get_next_target_time;
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::{VaultStatus, PostExecutionAction};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{BankMsg, Coin, DepsMut, Reply, Response};
use cosmwasm_std::{CosmosMsg, Env, Uint128, SubMsg, to_binary, WasmMsg};
use staking_router::msg::ExecuteMsg as StakingRouterExecuteMsg;

pub fn fin_limit_order_withdrawn_for_execute_vault(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let limit_order_cache = LIMIT_ORDER_CACHE.load(deps.storage)?;
    let vault = vault_store().load(deps.storage, cache.vault_id.into())?;

    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            remove_trigger(deps.storage, vault.id)?;

            save_trigger(
                deps.storage,
                Trigger {
                    vault_id: vault.id,
                    configuration: TriggerConfiguration::Time {
                        target_time: get_next_target_time(
                            env.block.time,
                            env.block.time,
                            vault.time_interval.clone(),
                        ),
                    },
                },
            )?;

            vault_store().update(
                deps.storage,
                vault.id.into(),
                |vault| -> Result<Vault, ContractError> {
                    match vault {
                        Some(mut existing_vault) => {
                            existing_vault.balance.amount -=
                                limit_order_cache.original_offer_amount;

                            if existing_vault.low_funds() {
                                existing_vault.status = VaultStatus::Inactive
                            }

                            if existing_vault.started_at.is_none() {
                                existing_vault.started_at = Some(env.block.time);
                            }

                            Ok(existing_vault)
                        }
                        None => Err(ContractError::CustomError {
                            val: format!(
                                "could not find vault for address: {} with id: {}",
                                cache.owner, cache.vault_id
                            ),
                        }),
                    }
                },
            )?;

            let coin_received = Coin {
                denom: vault.get_receive_denom().clone(),
                amount: limit_order_cache.filled,
            };

            let config = CONFIG.load(deps.storage)?;

            let execution_fee = Coin::new(
                (coin_received
                    .amount
                    .checked_multiply_ratio(config.fee_percent, ONE_HUNDRED)?)
                .into(),
                &coin_received.denom,
            );

            let mut messages: Vec<CosmosMsg> = Vec::new();
            let mut sub_msgs: Vec<SubMsg> = Vec::new();

            let total_to_redistribute = coin_received.amount - execution_fee.amount;

            vault.destinations.iter().for_each(|destination| {

                let amount = total_to_redistribute
                        .checked_multiply_ratio(
                            destination.allocation.atomics(),
                            Uint128::new(10)
                                .checked_pow(destination.allocation.decimal_places())
                                .unwrap(),
                        )
                        .unwrap();

                match destination.action {
                    PostExecutionAction::Send => {
                        messages.push(CosmosMsg::Bank(BankMsg::Send {
                            to_address: destination.address.to_string(),
                            amount: vec![Coin::new(
                                amount
                                    .u128(),
                                &coin_received.denom,
                            )],
                        }))
                    },
                    PostExecutionAction::ZDelegate => {
                        // authz delegations use funds from the users wallet so send back to user
                        messages.push(CosmosMsg::Bank(BankMsg::Send {
                            to_address: vault.owner.to_string(),
                            amount: vec![Coin::new(
                                amount.u128(),
                                &coin_received.denom,
                            )],
                        }));
                        sub_msgs.push(
                            SubMsg::reply_always(CosmosMsg::Wasm(WasmMsg::Execute {
                                contract_addr: config.staking_router_address.to_string(),
                                msg: to_binary(&StakingRouterExecuteMsg::ZDelegate {
                                    delegator_address: vault.owner.clone(),
                                    validator_address: destination.address.clone(),
                                    denom: vault.get_receive_denom(),
                                    amount,
                                })
                                .unwrap(),
                                funds: vec![],
                            }), DELEGATION_SUCCEEDED_ID)
                        )
                    }
                }
            });

            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: config.fee_collector.to_string(),
                amount: vec![execution_fee.clone()],
            }));

            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block,
                    EventData::DCAVaultExecutionCompleted {
                        sent: Coin {
                            denom: vault.get_swap_denom().clone(),
                            amount: limit_order_cache.original_offer_amount,
                        },
                        received: coin_received,
                        fee: execution_fee,
                    },
                ),
            )?;

            LIMIT_ORDER_CACHE.remove(deps.storage);
            //CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute(
                    "method",
                    "after_fin_limit_order_withdrawn_for_execute_trigger",
                )
                .add_attribute("vault_id", vault.id)
                .add_messages(messages)
                .add_submessages(sub_msgs)
            )
        }
        cosmwasm_std::SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!(
                "failed to withdraw fin limit order for vault id: {} - {}",
                vault.id, e
            ),
        }),
    }
}
