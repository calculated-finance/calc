use crate::error::ContractError;
use crate::helpers::vault_helpers::has_sufficient_funds;
use crate::state::events::create_event;
use crate::state::triggers::save_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::validation_helpers::{
    assert_contract_is_not_paused, assert_deposited_denom_matches_send_denom,
    assert_exactly_one_asset, assert_vault_is_not_cancelled,
};
use base::events::event::{EventBuilder, EventData};
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::VaultStatus;
use cosmwasm_std::{Addr, Env};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

use super::execute_trigger::execute_trigger;

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Addr,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    assert_contract_is_not_paused(deps.storage)?;
    deps.api.addr_validate(address.as_str())?;
    assert_exactly_one_asset(info.funds.clone())?;

    let mut vault = get_vault(deps.storage, vault_id.into())?;
    let vault_was_inactive = vault.is_inactive();

    if address != vault.owner {
        return Err(ContractError::CustomError {
            val: format!(
                "provided an incorrect owner address for vault id={:?}",
                vault_id
            ),
        });
    }

    assert_vault_is_not_cancelled(&vault)?;
    assert_deposited_denom_matches_send_denom(
        info.funds[0].denom.clone(),
        vault.clone().balance.denom,
    )?;

    vault.balance.amount += info.funds[0].amount;

    if !vault.is_scheduled() && has_sufficient_funds(vault.clone(), &deps.as_ref())? {
        vault.status = VaultStatus::Active
    }

    update_vault(deps.storage, &vault)?;

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

    let vault = get_vault(deps.storage, vault.id)?;

    let response = Response::new().add_attribute("method", "deposit");

    if vault.is_active() && vault_was_inactive {
        save_trigger(
            deps.storage,
            Trigger {
                vault_id,
                configuration: TriggerConfiguration::Time {
                    target_time: env.block.time.clone(),
                },
            },
        )?;

        return Ok(
            execute_trigger(deps, env, vault.id, response.clone()).expect("time trigger executed")
        );
    };

    Ok(response)
}
