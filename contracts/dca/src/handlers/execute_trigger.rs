use crate::contract::AFTER_FIN_SWAP_REPLY_ID;
use crate::error::ContractError;
use crate::helpers::fee_helpers::{get_delegation_fee_rate, get_swap_fee_rate};
use crate::helpers::validation_helpers::{
    assert_contract_is_not_paused, assert_target_time_is_in_past,
};
use crate::helpers::vault_helpers::get_swap_amount;
use crate::state::cache::{Cache, SwapCache, CACHE, SWAP_CACHE};
use crate::state::events::create_event;
use crate::state::triggers::{delete_trigger, save_trigger};
use crate::state::vaults::{get_vault, update_vault};
use base::events::event::{EventBuilder, EventData, ExecutionSkippedReason};
use base::helpers::time_helpers::get_next_target_time;
use base::price_type::PriceType;
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::VaultStatus;
use cosmwasm_std::{Coin, Decimal, ReplyOn};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Response, Uint128};
use fin_helpers::limit_orders::create_withdraw_limit_order_msg;
use fin_helpers::position_type::PositionType;
use fin_helpers::queries::{query_base_price, query_order_details, query_price, query_quote_price};
use fin_helpers::swaps::create_fin_swap_message;
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
    mut response: Response,
) -> Result<Response, ContractError> {
    let mut vault = get_vault(deps.storage, vault_id.into())?;

    delete_trigger(deps.storage, vault.id)?;

    if vault.is_cancelled() {
        return Err(ContractError::CustomError {
            val: format!(
                "vault with id {} is cancelled, and is not available for execution",
                vault.id
            ),
        });
    }

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
        }
        TriggerConfiguration::FinLimitOrder { order_idx, .. } => {
            if let Some(order_idx) = order_idx {
                let limit_order =
                    query_order_details(deps.querier, vault.pair.address.clone(), order_idx)?;

                if limit_order.offer_amount != Uint128::zero() {
                    return Err(ContractError::CustomError {
                        val: String::from("fin limit order has not been completely filled"),
                    });
                }

                if limit_order.filled_amount > Uint128::zero() {
                    response = response.add_message(create_withdraw_limit_order_msg(
                        vault.pair.address.clone(),
                        order_idx,
                    ));
                }
            } else {
                return Err(ContractError::CustomError {
                    val: String::from("fin limit order has not been created"),
                });
            }
        }
    }

    if !vault.has_sufficient_funds() {
        vault.status = VaultStatus::Inactive;
    }

    if vault.is_scheduled() {
        vault.status = VaultStatus::Active;
        vault.started_at = Some(env.block.time);
    }

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
                        vault.time_interval.clone(),
                    ),
                },
            },
        )?;
    }

    let position_type = vault.get_position_type();

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

    if vault.price_threshold_exceeded(fin_price) {
        if vault.is_active() {
            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block.to_owned(),
                    EventData::DcaVaultExecutionSkipped {
                        reason: ExecutionSkippedReason::PriceThresholdExceeded { price: fin_price },
                    },
                ),
            )?;
        }

        return Ok(response.to_owned());
    };

    if let Some(mut dca_plus_config) = vault.dca_plus_config.clone() {
        let swap_amount = min(dca_plus_config.total_deposit, vault.swap_amount);

        let price = query_price(
            deps.querier,
            vault.pair.clone(),
            &Coin::new(swap_amount.into(), vault.get_swap_denom()),
            PriceType::Actual,
        )?;

        let fee_rate = get_swap_fee_rate(&deps, &vault)? + get_delegation_fee_rate(&deps, &vault)?;
        let receive_amount = swap_amount * (Decimal::one() / price) * (Decimal::one() - fee_rate);

        dca_plus_config.standard_dca_swapped_amount += swap_amount;
        dca_plus_config.standard_dca_received_amount += receive_amount;

        if dca_plus_config.total_deposit - dca_plus_config.standard_dca_swapped_amount
            > Uint128::new(50000)
        {
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
                            vault.time_interval.clone(),
                        ),
                    },
                },
            )?;
        }

        vault.dca_plus_config = Some(dca_plus_config);
        update_vault(deps.storage, &vault)?;
    };

    if vault.is_active() {
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

        response = response.add_submessage(create_fin_swap_message(
            deps.querier,
            vault.pair.clone(),
            get_swap_amount(&deps.as_ref(), &env, vault.clone())?,
            vault.slippage_tolerance,
            Some(AFTER_FIN_SWAP_REPLY_ID),
            Some(ReplyOn::Always),
        )?);
    };

    Ok(response)
}
