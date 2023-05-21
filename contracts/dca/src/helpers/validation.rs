use crate::error::ContractError;
use crate::state::old_config::get_old_config;
use crate::state::pairs::get_pairs;
use crate::types::fee_collector::FeeCollector;
use crate::types::old_vault::OldVault;
use base::pair::OldPair;
use base::vaults::vault::{OldDestination, OldVaultStatus, PostExecutionAction};
use cosmwasm_std::{Addr, Coin, Decimal, Deps, Env, Storage, Timestamp, Uint128};

pub fn assert_exactly_one_asset(funds: Vec<Coin>) -> Result<(), ContractError> {
    if funds.is_empty() || funds.len() > 1 {
        return Err(ContractError::CustomError {
            val: format!("received {} denoms but required exactly 1", funds.len()),
        });
    }
    Ok(())
}

pub fn assert_contract_is_not_paused(storage: &mut dyn Storage) -> Result<(), ContractError> {
    let config = get_old_config(storage)?;
    if config.paused {
        return Err(ContractError::CustomError {
            val: "contract is paused".to_string(),
        });
    }
    Ok(())
}

pub fn assert_sender_is_admin(
    storage: &mut dyn Storage,
    sender: Addr,
) -> Result<(), ContractError> {
    let config = get_old_config(storage)?;
    if sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn assert_sender_is_executor(
    storage: &mut dyn Storage,
    env: &Env,
    sender: &Addr,
) -> Result<(), ContractError> {
    let config = get_old_config(storage)?;
    if !config.executors.contains(&sender)
        && sender != &config.admin
        && sender != &env.contract.address
    {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn asset_sender_is_vault_owner(vault_owner: Addr, sender: Addr) -> Result<(), ContractError> {
    if sender != vault_owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn assert_sender_is_admin_or_vault_owner(
    storage: &mut dyn Storage,
    vault_owner: Addr,
    sender: Addr,
) -> Result<(), ContractError> {
    let config = get_old_config(storage)?;
    if sender != config.admin && sender != vault_owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn assert_sender_is_contract_or_admin(
    storage: &mut dyn Storage,
    sender: &Addr,
    env: &Env,
) -> Result<(), ContractError> {
    let config = get_old_config(storage)?;
    if sender != &config.admin && sender != &env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn assert_vault_is_not_cancelled(vault: &OldVault) -> Result<(), ContractError> {
    if vault.status == OldVaultStatus::Cancelled {
        return Err(ContractError::CustomError {
            val: "vault is already cancelled".to_string(),
        });
    }
    Ok(())
}

pub fn assert_swap_amount_is_greater_than_50000(swap_amount: Uint128) -> Result<(), ContractError> {
    if swap_amount <= Uint128::from(50000u128) {
        return Err(ContractError::CustomError {
            val: String::from("swap amount must be greater than 50000"),
        });
    }
    Ok(())
}

pub fn assert_send_denom_is_in_pair_denoms(
    pair: OldPair,
    send_denom: String,
) -> Result<(), ContractError> {
    if send_denom != pair.base_denom && send_denom != pair.quote_denom {
        return Err(ContractError::CustomError {
            val: format!(
                "send denom {} does not match pair base denom {} or quote denom {}",
                send_denom, pair.base_denom, pair.quote_denom
            ),
        });
    }
    Ok(())
}

pub fn assert_deposited_denom_matches_send_denom(
    deposit_denom: String,
    send_denom: String,
) -> Result<(), ContractError> {
    if deposit_denom != send_denom {
        return Err(ContractError::CustomError {
            val: format!(
                "received asset with denom {}, but needed {}",
                deposit_denom, send_denom
            ),
        });
    }
    Ok(())
}

pub fn assert_target_start_time_is_in_future(
    current_time: Timestamp,
    target_start_time: Timestamp,
) -> Result<(), ContractError> {
    if current_time.seconds().gt(&target_start_time.seconds()) {
        return Err(ContractError::CustomError {
            val: String::from("target_start_time_utc_seconds must be some time in the future"),
        });
    }
    Ok(())
}

pub fn assert_target_time_is_in_past(
    current_time: Timestamp,
    target_time: Timestamp,
) -> Result<(), ContractError> {
    if current_time.seconds().lt(&target_time.seconds()) {
        return Err(ContractError::CustomError {
            val: String::from("trigger execution time has not yet elapsed"),
        });
    }
    Ok(())
}

pub fn assert_destinations_limit_is_not_breached(
    destinations: &[OldDestination],
) -> Result<(), ContractError> {
    if destinations.len() > 10 {
        return Err(ContractError::CustomError {
            val: String::from("no more than 10 destinations can be provided"),
        });
    };
    Ok(())
}

pub fn assert_destination_send_addresses_are_valid(
    deps: Deps,
    destinations: &[OldDestination],
) -> Result<(), ContractError> {
    for destination in destinations
        .iter()
        .filter(|d| d.action == PostExecutionAction::Send)
    {
        assert_address_is_valid(deps, destination.address.clone(), "destination".to_string())?;
    }
    Ok(())
}

pub fn assert_fee_collector_addresses_are_valid(
    deps: Deps,
    fee_collectors: &[FeeCollector],
) -> Result<(), ContractError> {
    for fee_collector in fee_collectors {
        match fee_collector.address.as_str() {
            "community_pool" => (),
            _ => assert_address_is_valid(
                deps,
                Addr::unchecked(fee_collector.address.clone()),
                "fee collector".to_string(),
            )?,
        }
    }
    Ok(())
}

pub fn assert_fee_level_is_valid(swap_fee_percent: &Decimal) -> Result<(), ContractError> {
    if swap_fee_percent > &Decimal::percent(5) {
        return Err(ContractError::CustomError {
            val: "fee level cannot be larger than 5%".to_string(),
        });
    }
    Ok(())
}

pub fn assert_risk_weighted_average_escrow_level_is_no_greater_than_100_percent(
    risk_weighted_average_escrow_level: Decimal,
) -> Result<(), ContractError> {
    if risk_weighted_average_escrow_level > Decimal::percent(100) {
        return Err(ContractError::CustomError {
            val: "risk_weighted_average_escrow_level cannot be greater than 100%".to_string(),
        });
    }
    Ok(())
}

pub fn assert_twap_period_is_valid(twap_period: u64) -> Result<(), ContractError> {
    if !(30..=3600).contains(&twap_period) {
        return Err(ContractError::CustomError {
            val: "twap_period must be between 30 and 3600".to_string(),
        });
    }
    Ok(())
}

pub fn assert_slippage_tolerance_is_less_than_or_equal_to_one(
    slippage_tolerance: Decimal,
) -> Result<(), ContractError> {
    if slippage_tolerance > Decimal::percent(100) {
        return Err(ContractError::CustomError {
            val: "default slippage tolerance must be less than or equal to 1".to_string(),
        });
    }
    Ok(())
}

pub fn assert_no_more_than_10_fee_collectors(
    fee_collectors: &[FeeCollector],
) -> Result<(), ContractError> {
    if fee_collectors.len() > 10 {
        return Err(ContractError::CustomError {
            val: String::from("no more than 10 fee collectors are allowed"),
        });
    }
    Ok(())
}

pub fn assert_destination_validator_addresses_are_valid(
    deps: Deps,
    destinations: &[OldDestination],
) -> Result<(), ContractError> {
    for destination in destinations
        .iter()
        .filter(|d| d.action == PostExecutionAction::ZDelegate)
    {
        assert_validator_is_valid(deps, destination.address.to_string())?;
    }
    Ok(())
}

pub fn assert_delegation_denom_is_stakeable(
    destinations: &[OldDestination],
    receive_denom: String,
) -> Result<(), ContractError> {
    if destinations
        .iter()
        .any(|d| d.action == PostExecutionAction::ZDelegate)
    {
        assert_denom_is_bond_denom(receive_denom)?;
    }
    Ok(())
}

pub fn assert_address_is_valid(
    deps: Deps,
    address: Addr,
    label: String,
) -> Result<(), ContractError> {
    match deps.api.addr_validate(&address.to_string()) {
        Ok(_) => Ok(()),
        Err(_) => Err(ContractError::CustomError {
            val: format!("{} address {} is invalid", label, address),
        }),
    }
}

pub fn assert_addresses_are_valid(
    deps: Deps,
    addresses: &[Addr],
    label: &str,
) -> Result<(), ContractError> {
    Ok(addresses
        .iter()
        .map(|address| assert_address_is_valid(deps, address.clone(), label.to_string()))
        .collect::<Result<(), ContractError>>()?)
}

pub fn assert_destination_allocations_add_up_to_one(
    destinations: &[OldDestination],
) -> Result<(), ContractError> {
    if destinations
        .iter()
        .fold(Decimal::zero(), |acc, destintation| {
            acc.checked_add(destintation.allocation).unwrap()
        })
        != Decimal::percent(100)
    {
        return Err(ContractError::CustomError {
            val: String::from("destination allocations must add up to 1"),
        });
    }
    Ok(())
}

pub fn assert_fee_collector_allocations_add_up_to_one(
    fee_collectors: &[FeeCollector],
) -> Result<(), ContractError> {
    if fee_collectors
        .iter()
        .fold(Decimal::zero(), |acc, fee_collector| {
            acc.checked_add(fee_collector.allocation).unwrap()
        })
        != Decimal::percent(100)
    {
        return Err(ContractError::CustomError {
            val: String::from("fee collector allocations must add up to 1"),
        });
    }
    Ok(())
}

pub fn assert_dca_plus_escrow_level_is_less_than_100_percent(
    dca_plus_escrow_level: Decimal,
) -> Result<(), ContractError> {
    if dca_plus_escrow_level > Decimal::percent(100) {
        return Err(ContractError::CustomError {
            val: "dca_plus_escrow_level cannot be greater than 100%".to_string(),
        });
    }
    Ok(())
}

pub fn assert_no_destination_allocations_are_zero(
    destinations: &[OldDestination],
) -> Result<(), ContractError> {
    if destinations.iter().any(|d| d.allocation.is_zero()) {
        return Err(ContractError::CustomError {
            val: String::from("all destination allocations must be greater than 0"),
        });
    }
    Ok(())
}

pub fn assert_page_limit_is_valid(limit: Option<u16>) -> Result<(), ContractError> {
    if let Some(limit) = limit {
        if limit < 30 {
            return Err(ContractError::CustomError {
                val: "limit cannot be less than 30.".to_string(),
            });
        } else if limit > 1000 {
            return Err(ContractError::CustomError {
                val: "limit cannot be greater than 1000.".to_string(),
            });
        }
    }
    Ok(())
}

pub fn assert_validator_is_valid(
    deps: Deps,
    validator_address: String,
) -> Result<(), ContractError> {
    let validator = deps.querier.query_validator(validator_address.clone()).ok();

    if validator.is_none() {
        return Err(ContractError::CustomError {
            val: format!("validator {} is invalid", validator_address),
        });
    }
    Ok(())
}

pub fn assert_denom_is_bond_denom(denom: String) -> Result<(), ContractError> {
    if denom.clone() != "ukuji".to_string() {
        return Err(ContractError::CustomError {
            val: format!("{} is not the bond denomination", denom),
        });
    }
    Ok(())
}

pub fn assert_denom_exists(storage: &dyn Storage, denom: String) -> Result<(), ContractError> {
    let pairs = get_pairs(storage);
    if !pairs.iter().any(|p| p.denoms().contains(&denom)) {
        return Err(ContractError::CustomError {
            val: format!("{} is not supported", denom),
        });
    }
    Ok(())
}
