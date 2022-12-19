use crate::contract::{
    AFTER_BANK_SWAP_REPLY_ID, AFTER_FIN_LIMIT_ORDER_WITHDRAWN_FOR_EXECUTE_VAULT_REPLY_ID,
    AFTER_FIN_SWAP_REPLY_ID, AFTER_Z_DELEGATION_REPLY_ID,
};
use crate::error::ContractError;
use crate::state::cache::{Cache, LimitOrderCache, CACHE, LIMIT_ORDER_CACHE};
use crate::state::config::{get_config, get_custom_fee};
use crate::state::events::create_event;
use crate::state::fin_limit_order_change_timestamp::FIN_LIMIT_ORDER_CHANGE_TIMESTAMP;
use crate::state::triggers::{delete_trigger, save_trigger};
use crate::state::vaults::{get_vault, update_vault};
use crate::types::vault::Vault;
use crate::validation_helpers::{assert_contract_is_not_paused, assert_target_time_is_in_past};
use base::events::event::{EventBuilder, EventData, ExecutionSkippedReason};
use base::helpers::coin_helpers::add_to_coin;
use base::helpers::math_helpers::checked_mul;
use base::helpers::message_helpers::{
    get_attribute_in_event, get_coin_from_display_formatted_coin, get_flat_map_for_event_type,
};
use base::helpers::time_helpers::get_next_target_time;
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::{PostExecutionAction, VaultStatus};
use cosmwasm_std::{
    to_binary, Attribute, BankMsg, Coin, CosmosMsg, Reply, StdError, StdResult, SubMsg,
    SubMsgResult, WasmMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Response, Uint128};
use fin_helpers::limit_orders::create_withdraw_limit_order_sub_msg;
use fin_helpers::position_type::PositionType;
use fin_helpers::queries::{query_base_price, query_order_details, query_quote_price};
use fin_helpers::swaps::create_fin_swap_message;
use staking_router::msg::ExecuteMsg;
use std::cmp::min;

pub fn execute_trigger_handler(
    deps: DepsMut,
    env: Env,
    trigger_id: Uint128,
) -> Result<Response, ContractError> {
    assert_contract_is_not_paused(deps.storage)?;
    let response = Response::new().add_attribute("method", "execute_trigger");
    Ok(execute_trigger(deps, env, trigger_id, response)?)
}

pub fn execute_trigger(
    deps: DepsMut,
    env: Env,
    vault_id: Uint128,
    response: Response,
) -> Result<Response, ContractError> {
    let vault = get_vault(deps.storage, vault_id.into())?;

    let position_type = vault.get_position_type();

    if vault.is_scheduled() {
        update_vault(deps.storage, vault.id, |stored_value| match stored_value {
            Some(mut existing_vault) => {
                existing_vault.status = VaultStatus::Active;
                existing_vault.started_at = Some(env.block.time);
                Ok(existing_vault)
            }
            None => Err(StdError::NotFound {
                kind: format!(
                    "vault for address: {} with id: {}",
                    vault.owner.clone(),
                    vault.id
                ),
            }),
        })?;
    }

    let fin_price = match position_type {
        PositionType::Enter => query_base_price(deps.querier, vault.pair.address.clone()),
        PositionType::Exit => query_quote_price(deps.querier, vault.pair.address.clone()),
    };

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.to_owned(),
            EventData::DcaVaultExecutionTriggered {
                base_denom: vault.pair.base_denom.clone(),
                quote_denom: vault.pair.quote_denom.clone(),
                asset_price: fin_price.clone(),
            },
        ),
    )?;

    if vault.trigger.is_none() {
        return Err(ContractError::CustomError {
            val: format!(
                "vault with id {} has no trigger attached, and is not available for execution",
                vault.id
            ),
        });
    }

    match vault
        .trigger
        .clone()
        .expect(format!("trigger for vault id {}", vault.id).as_str())
    {
        TriggerConfiguration::Time { target_time } => {
            assert_target_time_is_in_past(env.block.time, target_time)?;

            if vault.price_threshold_exceeded(fin_price) {
                create_event(
                    deps.storage,
                    EventBuilder::new(
                        vault.id,
                        env.block.to_owned(),
                        EventData::DcaVaultExecutionSkipped {
                            reason: ExecutionSkippedReason::PriceThresholdExceeded {
                                price: fin_price,
                            },
                        },
                    ),
                )?;

                delete_trigger(deps.storage, vault.id)?;

                save_trigger(
                    deps.storage,
                    Trigger {
                        vault_id: vault.id,
                        configuration: TriggerConfiguration::Time {
                            target_time: get_next_target_time(
                                env.block.time,
                                target_time,
                                vault.time_interval.clone(),
                            ),
                        },
                    },
                )?;

                return Ok(response.to_owned());
            };

            CACHE.save(
                deps.storage,
                &Cache {
                    vault_id: vault.id,
                    owner: vault.owner.clone(),
                },
            )?;

            return Ok(response.add_submessage(create_fin_swap_message(
                deps.querier,
                vault.pair.address.clone(),
                vault.get_swap_amount(),
                vault.get_position_type(),
                vault.slippage_tolerance,
                AFTER_FIN_SWAP_REPLY_ID,
            )));
        }
        TriggerConfiguration::FinLimitOrder { order_idx, .. } => {
            if let Some(order_idx) = order_idx {
                let limit_order_details =
                    query_order_details(deps.querier, vault.pair.address.clone(), order_idx)?;

                let limit_order_cache = LimitOrderCache {
                    order_idx,
                    offer_amount: limit_order_details.offer_amount,
                    original_offer_amount: limit_order_details.original_offer_amount,
                    filled: limit_order_details.filled_amount,
                    quote_price: limit_order_details.quote_price,
                    created_at: limit_order_details.created_at,
                };

                LIMIT_ORDER_CACHE.save(deps.storage, &limit_order_cache)?;

                if limit_order_cache.offer_amount != Uint128::zero() {
                    return Err(ContractError::CustomError {
                        val: String::from("fin limit order has not been completely filled"),
                    });
                }

                let fin_withdraw_sub_msg = create_withdraw_limit_order_sub_msg(
                    vault.pair.address,
                    order_idx,
                    AFTER_FIN_LIMIT_ORDER_WITHDRAWN_FOR_EXECUTE_VAULT_REPLY_ID,
                );

                let cache: Cache = Cache {
                    vault_id: vault.id,
                    owner: vault.owner.clone(),
                };

                CACHE.save(deps.storage, &cache)?;

                return Ok(response.add_submessage(fin_withdraw_sub_msg));
            } else {
                return Err(ContractError::CustomError {
                    val: String::from("fin limit order has not been created"),
                });
            }
        }
    }
}

pub fn after_fin_swap(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id.into())?;

    let mut attributes: Vec<Attribute> = Vec::new();
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut sub_msgs: Vec<SubMsg> = Vec::new();

    delete_trigger(deps.storage, vault.id)?;

    match reply.result {
        SubMsgResult::Ok(_) => {
            let fin_swap_response = reply.result.into_result().unwrap();

            let wasm_trade_event =
                get_flat_map_for_event_type(&fin_swap_response.events, "wasm-trade").unwrap();

            let base_amount = wasm_trade_event["base_amount"].parse::<u128>().unwrap();
            let quote_amount = wasm_trade_event["quote_amount"].parse::<u128>().unwrap();

            let (coin_sent, coin_received) = match vault.get_position_type() {
                PositionType::Enter => {
                    let sent = Coin {
                        denom: vault.get_swap_denom(),
                        amount: Uint128::from(quote_amount),
                    };
                    let received = Coin {
                        denom: vault.get_receive_denom(),
                        amount: Uint128::from(base_amount),
                    };

                    (sent, received)
                }
                PositionType::Exit => {
                    let sent = Coin {
                        denom: vault.get_swap_denom(),
                        amount: Uint128::from(base_amount),
                    };
                    let received = Coin {
                        denom: vault.get_receive_denom(),
                        amount: Uint128::from(quote_amount),
                    };

                    (sent, received)
                }
            };

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

            if swap_fee.gt(&Uint128::zero()) {
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.fee_collector.to_string(),
                    amount: vec![Coin::new(swap_fee.into(), coin_received.denom.clone())],
                }));
            }

            if automation_fee.gt(&Uint128::zero()) {
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.fee_collector.to_string(),
                    amount: vec![Coin::new(
                        automation_fee.into(),
                        coin_received.denom.clone(),
                    )],
                }));
            }

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
                                    msg: to_binary(&ExecuteMsg::ZDelegate {
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

            let updated_vault = update_vault(
                deps.storage,
                vault.id.into(),
                |stored_value: Option<Vault>| -> StdResult<Vault> {
                    match stored_value {
                        Some(mut existing_vault) => {
                            existing_vault.balance.amount -=
                                existing_vault.get_swap_amount().amount;

                            if !existing_vault.has_sufficient_funds() {
                                existing_vault.status = VaultStatus::Inactive;
                            }

                            existing_vault.swapped_amount =
                                add_to_coin(existing_vault.swapped_amount, coin_sent.amount)?;

                            existing_vault.received_amount =
                                add_to_coin(existing_vault.received_amount, total_after_total_fee)?;

                            Ok(existing_vault)
                        }
                        None => Err(StdError::NotFound {
                            kind: format!(
                                "vault for address: {} with id: {}",
                                vault.owner.clone(),
                                vault.id
                            ),
                        }),
                    }
                },
            )?;

            if updated_vault.is_active() {
                match vault
                    .trigger
                    .expect(format!("trigger for vault id {}", vault.id).as_str())
                {
                    TriggerConfiguration::Time { target_time } => {
                        save_trigger(
                            deps.storage,
                            Trigger {
                                vault_id: vault.id,
                                configuration: TriggerConfiguration::Time {
                                    target_time: get_next_target_time(
                                        env.block.time,
                                        target_time,
                                        vault.time_interval,
                                    ),
                                },
                            },
                        )?;
                    }
                    TriggerConfiguration::FinLimitOrder { .. } => {
                        save_trigger(
                            deps.storage,
                            Trigger {
                                vault_id: vault.id,
                                configuration: TriggerConfiguration::Time {
                                    target_time: get_next_target_time(
                                        env.block.time,
                                        env.block.time,
                                        vault.time_interval,
                                    ),
                                },
                            },
                        )?;
                    }
                }
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
            if !vault.has_sufficient_funds() {
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

                update_vault(
                    deps.storage,
                    vault.id.into(),
                    |existing_vault| -> StdResult<Vault> {
                        match existing_vault {
                            Some(mut existing_vault) => {
                                existing_vault.status = VaultStatus::Inactive;
                                Ok(existing_vault)
                            }
                            None => Err(StdError::NotFound {
                                kind: format!(
                                    "vault for address: {} with id: {}",
                                    vault.owner.clone(),
                                    vault.id
                                ),
                            }),
                        }
                    },
                )?;
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

                match vault
                    .trigger
                    .expect(format!("trigger for vault id {}", vault.id).as_str())
                {
                    TriggerConfiguration::Time { target_time } => {
                        save_trigger(
                            deps.storage,
                            Trigger {
                                vault_id: vault.id,
                                configuration: TriggerConfiguration::Time {
                                    target_time: get_next_target_time(
                                        env.block.time,
                                        target_time,
                                        vault.time_interval,
                                    ),
                                },
                            },
                        )?;
                    }
                    _ => panic!("should be a time trigger"),
                }
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

pub fn after_fin_limit_order_withdrawn_for_execute_vault(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let limit_order_cache = LIMIT_ORDER_CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id.into())?;

    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            let mut messages: Vec<CosmosMsg> = Vec::new();
            let mut sub_msgs: Vec<SubMsg> = Vec::new();

            let withdraw_order_response = reply.result.into_result().unwrap();

            let received_amount =
                get_attribute_in_event(&withdraw_order_response.events, "transfer", "amount")?
                    .trim_end_matches(&vault.get_receive_denom().to_string())
                    .parse::<Uint128>()
                    .expect("limit order withdrawn amount");

            let coin_received = Coin {
                denom: vault.get_receive_denom().clone(),
                amount: received_amount,
            };

            let config = get_config(deps.storage)?;

            let fin_limit_order_change_timestamp =
                FIN_LIMIT_ORDER_CHANGE_TIMESTAMP.may_load(deps.storage)?;

            let is_new_fin_limit_order = fin_limit_order_change_timestamp.is_some()
                && limit_order_cache.created_at > fin_limit_order_change_timestamp.unwrap();

            if is_new_fin_limit_order {
                if coin_received.amount.gt(&Uint128::zero()) {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: config.fee_collector.to_string(),
                        amount: vec![coin_received],
                    }));
                }

                sub_msgs.push(create_fin_swap_message(
                    deps.querier,
                    vault.pair.address.clone(),
                    vault.get_swap_amount(),
                    vault.get_position_type(),
                    vault.slippage_tolerance,
                    AFTER_FIN_SWAP_REPLY_ID,
                ));
            } else {
                delete_trigger(deps.storage, vault.id)?;

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

                if swap_fee.gt(&Uint128::zero()) {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: config.fee_collector.to_string(),
                        amount: vec![Coin::new(swap_fee.into(), coin_received.denom.clone())],
                    }));
                }

                if automation_fee.gt(&Uint128::zero()) {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: config.fee_collector.to_string(),
                        amount: vec![Coin::new(
                            automation_fee.into(),
                            coin_received.denom.clone(),
                        )],
                    }));
                }

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
                                        msg: to_binary(&ExecuteMsg::ZDelegate {
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

                let updated_vault = update_vault(
                    deps.storage,
                    vault.id.into(),
                    |stored_value: Option<Vault>| -> StdResult<Vault> {
                        match stored_value {
                            Some(mut existing_vault) => {
                                existing_vault.balance.amount -=
                                    limit_order_cache.original_offer_amount;

                                if !existing_vault.has_sufficient_funds() {
                                    existing_vault.status = VaultStatus::Inactive
                                }

                                existing_vault.swapped_amount = add_to_coin(
                                    existing_vault.swapped_amount,
                                    limit_order_cache.original_offer_amount,
                                )?;

                                existing_vault.received_amount = add_to_coin(
                                    existing_vault.received_amount,
                                    total_after_total_fee,
                                )?;

                                Ok(existing_vault)
                            }
                            None => Err(StdError::NotFound {
                                kind: format!(
                                    "vault for address: {} with id: {}",
                                    vault.owner.clone(),
                                    vault.id
                                ),
                            }),
                        }
                    },
                )?;

                if updated_vault.is_active() {
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
                }

                create_event(
                    deps.storage,
                    EventBuilder::new(
                        vault.id,
                        env.block,
                        EventData::DcaVaultExecutionCompleted {
                            sent: Coin {
                                denom: vault.get_swap_denom().clone(),
                                amount: limit_order_cache.original_offer_amount,
                            },
                            received: coin_received.clone(),
                            fee: Coin::new(total_fee.into(), coin_received.denom),
                        },
                    ),
                )?;
            }

            Ok(Response::new()
                .add_attribute("method", "fin_limit_order_withdrawn_for_execute_vault")
                .add_attribute("vault_id", vault.id)
                .add_messages(messages)
                .add_submessages(sub_msgs))
        }
        cosmwasm_std::SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!(
                "failed to withdraw fin limit order for vault id: {} - {}",
                vault.id, e
            ),
        }),
    }
}

pub fn after_z_delegation(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id.into())?;

    match reply.result {
        SubMsgResult::Ok(_) => {
            let z_delegate_response = reply.result.into_result().unwrap();

            let validator_address =
                get_attribute_in_event(&z_delegate_response.events, "delegate", "validator")?;

            let display_formatted_coin =
                get_attribute_in_event(&z_delegate_response.events, "delegate", "amount")?;

            let delegation = get_coin_from_display_formatted_coin(display_formatted_coin);

            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block,
                    EventData::DcaVaultZDelegationSucceeded {
                        validator_address,
                        delegation,
                    },
                ),
            )?;
        }
        SubMsgResult::Err(_) => {
            create_event(
                deps.storage,
                EventBuilder::new(vault.id, env.block, EventData::DcaVaultDelegationFailed {}),
            )?;
        }
    }

    Ok(Response::new()
        .add_attribute("method", "after_z_delegation")
        .add_attribute("vault_id", vault.id.to_string()))
}
