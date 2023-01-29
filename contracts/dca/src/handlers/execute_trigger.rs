use crate::contract::{
    AFTER_FIN_LIMIT_ORDER_WITHDRAWN_FOR_EXECUTE_VAULT_REPLY_ID, AFTER_FIN_SWAP_REPLY_ID,
};
use crate::error::ContractError;
use crate::message_helpers::{swap_for_bow_deposit_messages, unstake_from_bow_message};
use crate::msg::ExecuteMsg;
use crate::state::cache::{
    BowCache, Cache, LimitOrderCache, SwapCache, BOW_CACHE, CACHE, LIMIT_ORDER_CACHE, SOURCE_CACHE,
    SWAP_CACHE,
};
use crate::state::events::create_event;
use crate::state::sources::{remove_source, save_source};
use crate::state::triggers::{delete_trigger, save_trigger};
use crate::state::vaults::{get_vault, update_vault};
use crate::types::source::Source;
use crate::validation_helpers::{assert_contract_is_not_paused, assert_target_time_is_in_past};
use base::events::event::{EventBuilder, EventData, ExecutionSkippedReason};
use base::helpers::time_helpers::get_next_target_time;
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::VaultStatus;
use cosmwasm_std::{to_binary, CosmosMsg, StdError, SubMsg, WasmMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Response, Uint128};
use fin_helpers::limit_orders::create_withdraw_limit_order_sub_msg;
use fin_helpers::position_type::PositionType;
use fin_helpers::queries::{query_base_price, query_order_details, query_quote_price};
use fin_helpers::swaps::create_fin_swap_message;

pub fn execute_trigger_handler(
    deps: &mut DepsMut,
    env: &Env,
    trigger_id: Uint128,
) -> Result<Response, ContractError> {
    assert_contract_is_not_paused(deps.storage)?;
    let vault = get_vault(deps.storage, trigger_id.into())?;

    if vault.trigger.is_none() {
        return Err(ContractError::CustomError {
            val: format!(
                "vault with id {} has no trigger attached, and is not available for execution",
                vault.id
            ),
        });
    }

    if vault.is_scheduled() {
        update_vault(deps.storage, vault.id, |stored_vault| {
            if let Some(mut stored_vault) = stored_vault {
                stored_vault.status = VaultStatus::Active;
                stored_vault.started_at = Some(env.block.time);
                return Ok(stored_vault);
            }
            Err(StdError::NotFound {
                kind: format!("Vault {}", vault.id),
            })
        })?;
    }

    let fin_price = match vault.get_position_type() {
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

    if let Some(trigger) = vault.trigger.clone() {
        if let TriggerConfiguration::Time { target_time } = trigger {
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

                return Ok(Response::new());
            };
        }
    }

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut sub_messages: Vec<SubMsg> = Vec::new();

    if let Some(source) = vault.source.clone() {
        match source.clone() {
            Source::Bow { address } => {
                BOW_CACHE.save(
                    deps.storage,
                    &BowCache {
                        pool_address: address,
                        lp_token_balance: Some(vault.balance.clone()),
                        deposit: vec![],
                        withdrawal: vec![],
                    },
                )?;

                sub_messages.push(unstake_from_bow_message(deps, env, &vault)?);
            }
        }

        SOURCE_CACHE.save(deps.storage, &source)?;
        remove_source(deps.storage, vault.id)?;

        return Ok(Response::new()
            .add_submessages(sub_messages)
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::ExecuteTrigger { trigger_id })?,
                funds: vec![],
            })));
    }

    match vault
        .trigger
        .clone()
        .expect(format!("trigger for vault id {}", vault.id).as_str())
    {
        TriggerConfiguration::Time { .. } => {
            CACHE.save(
                deps.storage,
                &Cache {
                    vault_id: vault.id,
                    owner: vault.owner.clone(),
                },
            )?;

            SWAP_CACHE.save(
                deps.storage,
                &SwapCache {
                    swap_denom_balance: deps
                        .querier
                        .query_balance(&env.contract.address, &vault.get_swap_denom())?,
                    receive_denom_balance: deps
                        .querier
                        .query_balance(&env.contract.address, &vault.get_receive_denom())?,
                },
            )?;

            sub_messages.push(SubMsg::reply_always(
                create_fin_swap_message(
                    deps.querier,
                    vault.pair.clone(),
                    vault.get_swap_amount(),
                    vault.slippage_tolerance,
                )?,
                AFTER_FIN_SWAP_REPLY_ID,
            ));
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
                    swap_denom_balance: deps
                        .querier
                        .query_balance(&env.contract.address, &vault.get_swap_denom())?,
                    receive_denom_balance: deps
                        .querier
                        .query_balance(&env.contract.address, &vault.get_receive_denom())?,
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

                let cache = Cache {
                    vault_id: vault.id,
                    owner: vault.owner.clone(),
                };

                CACHE.save(deps.storage, &cache)?;

                sub_messages.push(fin_withdraw_sub_msg);
            } else {
                return Err(ContractError::CustomError {
                    val: String::from("fin limit order has not been created"),
                });
            }
        }
    }

    if let Some(source) = SOURCE_CACHE.may_load(deps.storage)? {
        save_source(deps.storage, vault.id, source.clone())?;
        SOURCE_CACHE.remove(deps.storage);

        match source {
            Source::Bow { address, .. } => {
                messages.append(&mut swap_for_bow_deposit_messages(
                    deps,
                    env,
                    &address,
                    vault.balance.clone(),
                    vault.slippage_tolerance,
                )?);
            }
        };
    }

    Ok(Response::new()
        .add_attribute("method", "execute_trigger")
        .add_submessages(sub_messages)
        .add_messages(messages))
}
