use crate::constants::TWO_MICRONS;
use crate::contract::AFTER_FIN_LIMIT_ORDER_SUBMITTED_REPLY_ID;
use crate::error::ContractError;
use crate::state::cache::{Cache, CACHE};
use crate::state::events::create_event;
use crate::state::fin_limit_order_change_timestamp::FIN_LIMIT_ORDER_CHANGE_TIMESTAMP;
use crate::state::pairs::PAIRS;
use crate::state::triggers::save_trigger;
use crate::state::vaults::{save_vault, update_vault};
use crate::types::vault::Vault;
use crate::types::vault_builder::VaultBuilder;
use crate::validation_helpers::{
    assert_address_is_valid, assert_contract_is_not_paused, assert_delegation_denom_is_stakeable,
    assert_destination_allocations_add_up_to_one, assert_destination_send_addresses_are_valid,
    assert_destination_validator_addresses_are_valid, assert_destinations_limit_is_not_breached,
    assert_exactly_one_asset, assert_no_destination_allocations_are_zero,
    assert_send_denom_is_in_pair_denoms, assert_swap_amount_is_greater_than_50000,
    assert_target_start_time_is_in_future,
};
use base::events::event::{EventBuilder, EventData};
use base::triggers::trigger::{TimeInterval, Trigger, TriggerConfiguration};
use base::vaults::vault::{Destination, PostExecutionAction, VaultStatus};
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, StdError};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Timestamp, Uint128, Uint64};
use fin_helpers::limit_orders::create_limit_order_sub_msg;
use fin_helpers::position_type::PositionType;
use fin_helpers::queries::query_pair_config;

use super::execute_trigger::execute_trigger;

pub fn create_vault(
    mut deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    owner: Addr,
    label: Option<String>,
    mut destinations: Vec<Destination>,
    pair_address: Addr,
    position_type: Option<PositionType>,
    slippage_tolerance: Option<Decimal256>,
    minimum_receive_amount: Option<Uint128>,
    swap_amount: Uint128,
    time_interval: TimeInterval,
    target_start_time_utc_seconds: Option<Uint64>,
    target_receive_amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    assert_contract_is_not_paused(deps.storage)?;
    assert_address_is_valid(deps.as_ref(), owner.clone(), "owner".to_string())?;
    assert_exactly_one_asset(info.funds.clone())?;
    assert_swap_amount_is_greater_than_50000(swap_amount)?;
    assert_destinations_limit_is_not_breached(&destinations)?;

    if let Some(target_time) = target_start_time_utc_seconds {
        assert_target_start_time_is_in_future(
            env.block.time,
            Timestamp::from_seconds(target_time.u64()),
        )?;
    }

    if destinations.is_empty() {
        destinations.push(Destination {
            address: owner.clone(),
            allocation: Decimal::percent(100),
            action: PostExecutionAction::Send,
        });
    }

    assert_destination_send_addresses_are_valid(deps.as_ref(), &destinations)?;
    assert_destination_validator_addresses_are_valid(deps.as_ref(), &destinations)?;
    assert_no_destination_allocations_are_zero(&destinations)?;
    assert_destination_allocations_add_up_to_one(&destinations)?;

    deps.api.addr_validate(&pair_address.to_string())?;
    let pair = PAIRS.load(deps.storage, pair_address)?;

    let send_denom = info.funds[0].denom.clone();

    assert_send_denom_is_in_pair_denoms(pair.clone(), send_denom.clone())?;

    let receive_denom = if send_denom == pair.quote_denom {
        pair.base_denom.clone()
    } else {
        pair.quote_denom.clone()
    };

    assert_delegation_denom_is_stakeable(&destinations, receive_denom)?;

    let vault_builder = VaultBuilder {
        owner,
        label,
        destinations,
        created_at: env.block.time,
        status: if info.funds[0].amount.clone() <= Uint128::from(50000u128) {
            VaultStatus::Inactive
        } else {
            VaultStatus::Scheduled
        },
        pair,
        swap_amount,
        position_type,
        slippage_tolerance,
        minimum_receive_amount,
        balance: info.funds[0].clone(),
        time_interval: time_interval.clone(),
        started_at: None,
    };

    let vault = save_vault(deps.storage, vault_builder)?;

    CACHE.save(
        deps.storage,
        &Cache {
            vault_id: vault.id.clone(),
            owner: vault.owner.clone(),
        },
    )?;

    create_event(
        deps.storage,
        EventBuilder::new(vault.id, env.block.clone(), EventData::DcaVaultCreated {}),
    )?;

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.clone(),
            EventData::DcaVaultFundsDeposited {
                amount: info.funds[0].clone(),
            },
        ),
    )?;

    let response = Response::new()
        .add_attribute("method", "create_vault")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id);

    if vault.is_inactive() {
        return Ok(response);
    }

    match (target_start_time_utc_seconds, target_receive_amount) {
        (None, None) | (Some(_), None) => {
            let response = create_time_trigger(
                &mut deps,
                &env,
                &vault,
                target_start_time_utc_seconds,
                &response,
            )
            .expect("time trigger created");

            if target_start_time_utc_seconds.is_none() {
                return Ok(
                    execute_trigger(deps, env, vault.id, response).expect("time trigger executed")
                );
            }

            Ok(response)
        }
        (None, Some(target_receive_amount)) => {
            create_fin_limit_order_trigger(deps, vault, target_receive_amount, response)
        }
        (Some(_), Some(_)) => Err(ContractError::CustomError {
            val: String::from(
                "cannot provide both a target_start_time_utc_seconds and a target_price",
            ),
        }),
    }
}

fn create_time_trigger(
    deps: &mut DepsMut,
    env: &Env,
    vault: &Vault,
    target_start_time_utc_seconds: Option<Uint64>,
    response: &Response,
) -> Result<Response, ContractError> {
    let target_time: Timestamp = match target_start_time_utc_seconds {
        Some(time) => Timestamp::from_seconds(time.u64()),
        None => env.block.time,
    };

    save_trigger(
        deps.storage,
        Trigger {
            vault_id: vault.id,
            configuration: TriggerConfiguration::Time { target_time },
        },
    )?;

    Ok(response.to_owned())
}

fn create_fin_limit_order_trigger(
    deps: DepsMut,
    vault: Vault,
    target_receive_amount: Uint128,
    response: Response,
) -> Result<Response, ContractError> {
    let pair_config = query_pair_config(deps.querier, vault.pair.address.clone())?;

    let target_price = vault.get_target_price(
        target_receive_amount,
        pair_config.decimal_delta.unwrap_or(0),
        pair_config.price_precision,
    )?;

    save_trigger(
        deps.storage,
        Trigger {
            vault_id: vault.id,
            configuration: TriggerConfiguration::FinLimitOrder {
                order_idx: None,
                target_price,
            },
        },
    )?;

    let fin_limit_order_change_timestamp =
        FIN_LIMIT_ORDER_CHANGE_TIMESTAMP.may_load(deps.storage)?;

    let is_new_fin_limit_order = fin_limit_order_change_timestamp.is_some();

    if is_new_fin_limit_order {
        update_vault(deps.storage, vault.id, |stored_vault| match stored_vault {
            Some(mut vault) => {
                vault.balance.amount -= TWO_MICRONS;
                Ok(vault)
            }
            None => Err(StdError::GenericErr {
                msg: format!("Vault ({}) not found", vault.id).to_string(),
            }),
        })?;
    }

    let fin_limit_order_sub_msg = create_limit_order_sub_msg(
        vault.pair.address.clone(),
        target_price,
        if is_new_fin_limit_order {
            Coin::new(TWO_MICRONS.into(), vault.get_swap_denom())
        } else {
            vault.get_swap_amount()
        },
        AFTER_FIN_LIMIT_ORDER_SUBMITTED_REPLY_ID,
    );

    Ok(response.add_submessage(fin_limit_order_sub_msg))
}
