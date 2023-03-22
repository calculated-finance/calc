use crate::contract::AFTER_FIN_SWAP_REPLY_ID;
use crate::error::ContractError;
use crate::helpers::fee_helpers::{get_delegation_fee_rate, get_swap_fee_rate};
use crate::helpers::validation_helpers::{
    assert_contract_is_not_paused, assert_target_time_is_in_past,
};
use crate::helpers::vault_helpers::get_swap_amount;
use crate::msg::ExecuteMsg;
use crate::state::cache::{Cache, SwapCache, CACHE, SWAP_CACHE};
use crate::state::events::create_event;
use crate::state::triggers::{delete_trigger, save_trigger};
use crate::state::vaults::{get_vault, update_vault};
use base::events::event::{EventBuilder, EventData, ExecutionSkippedReason};
use base::helpers::coin_helpers::add_to_coin;
use base::helpers::time_helpers::get_next_target_time;
use base::price_type::PriceType;
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::VaultStatus;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, Decimal, ReplyOn, StdResult, WasmMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Response, Uint128};
use fin_helpers::limit_orders::create_withdraw_limit_order_msg;
use fin_helpers::queries::{query_order_details, query_price};
use fin_helpers::swaps::create_fin_swap_message;
use osmosis_helpers::queries::query_quote_price;
use osmosis_helpers::swap::create_osmosis_swap_message;
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
        } // TriggerConfiguration::FinLimitOrder { order_idx, .. } => {
          //     if let Some(order_idx) = order_idx {
          //         let limit_order =
          //             query_order_details(deps.querier, vault.pool.address.clone(), order_idx)?;

          //         if limit_order.offer_amount != Uint128::zero() {
          //             return Err(ContractError::CustomError {
          //                 val: String::from("fin limit order has not been completely filled"),
          //             });
          //         }

          //         if limit_order.filled_amount > Uint128::zero() {
          //             response = response.add_message(create_withdraw_limit_order_msg(
          //                 vault.pool.address.clone(),
          //                 order_idx,
          //             ));
          //         }
          //     } else {
          //         return Err(ContractError::CustomError {
          //             val: String::from("fin limit order has not been created"),
          //         });
          //     }
          // }
    }

    if vault.is_scheduled() {
        vault.status = VaultStatus::Active;
        vault.started_at = Some(env.block.time);
    }

    if !vault.has_sufficient_funds() {
        vault.status = VaultStatus::Inactive;
    }

    let standard_dca_still_active = vault.dca_plus_config.clone().map_or(
        Ok(false),
        |mut dca_plus_config| -> StdResult<bool> {
            let swap_amount = min(
                dca_plus_config.clone().standard_dca_balance().amount,
                vault.swap_amount,
            );

            let price =
                query_quote_price(deps.querier, &vault.pool.clone(), &vault.get_swap_denom())?;

            let fee_rate =
                get_swap_fee_rate(&deps, &vault)? + get_delegation_fee_rate(&deps, &vault)?;

            let receive_amount =
                swap_amount * (Decimal::one() / price) * (Decimal::one() - fee_rate);

            dca_plus_config.standard_dca_swapped_amount =
                add_to_coin(dca_plus_config.standard_dca_swapped_amount, swap_amount);

            dca_plus_config.standard_dca_received_amount =
                add_to_coin(dca_plus_config.standard_dca_received_amount, receive_amount);

            vault.dca_plus_config = Some(dca_plus_config.clone());

            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block.clone(),
                    EventData::DcaVaultExecutionCompleted {
                        sent: Coin::new(swap_amount.into(), vault.get_swap_denom()),
                        received: Coin::new(receive_amount.into(), vault.get_receive_denom()),
                        fee: Coin::new((fee_rate * receive_amount).into(), vault.get_swap_denom()),
                    },
                ),
            )?;

            Ok(dca_plus_config.has_sufficient_funds())
        },
    )?;

    update_vault(deps.storage, &vault)?;

    if vault.is_active() || (vault.is_dca_plus() && standard_dca_still_active) {
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

    if !vault.is_active() {
        if vault.is_dca_plus() && !standard_dca_still_active {
            response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::DisburseEscrow { vault_id: vault.id })?,
                funds: vec![],
            }))
        }

        return Ok(response.to_owned());
    }

    let fin_price = query_quote_price(deps.querier, &vault.pool, &vault.get_swap_denom())?;

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.to_owned(),
            EventData::DcaVaultExecutionTriggered {
                base_denom: vault.pool.base_denom.clone(),
                quote_denom: vault.pool.quote_denom.clone(),
                asset_price: fin_price.clone(),
            },
        ),
    )?;

    if vault.price_threshold_exceeded(fin_price) {
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

        return Ok(response.to_owned());
    };

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

    Ok(response.add_submessage(create_osmosis_swap_message(
        deps.querier,
        env.contract.address.clone().into(),
        vault.pool.clone(),
        get_swap_amount(&deps.as_ref(), &env, vault.clone())?,
        vault.slippage_tolerance,
        Some(AFTER_FIN_SWAP_REPLY_ID),
        Some(ReplyOn::Always),
    )?))
}
