use crate::error::ContractError;
use crate::message_helpers::{execute_trigger_message, swap_for_bow_deposit_messages};
use crate::state::events::create_event;
use crate::state::triggers::save_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::types::source::Source;
use crate::validation_helpers::{
    assert_contract_is_not_paused, assert_deposited_denom_matches_send_denom,
    assert_number_of_assets_equals, assert_vault_is_not_cancelled,
};
use base::events::event::{EventBuilder, EventData};
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::VaultStatus;
use cosmwasm_std::{Addr, CosmosMsg, Env, StdError};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

pub fn deposit(
    deps: &mut DepsMut,
    env: Env,
    info: MessageInfo,
    address: Addr,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    assert_contract_is_not_paused(deps.storage)?;
    deps.api.addr_validate(address.as_str())?;
    assert_number_of_assets_equals(info.funds.clone(), 1)?;

    let vault = get_vault(deps.storage, vault_id.into())?;

    if address != vault.owner {
        return Err(ContractError::CustomError {
            val: format!(
                "provided an incorrect owner address for vault id={:?}",
                vault_id
            ),
        });
    }

    let deposit = info.funds[0].clone();

    assert_vault_is_not_cancelled(&vault)?;
    assert_deposited_denom_matches_send_denom(deposit.denom.clone(), vault.balance.denom.clone())?;

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.clone(),
            EventData::DcaVaultFundsDeposited {
                amount: deposit.clone(),
            },
        ),
    )?;

    let mut messages = Vec::<CosmosMsg>::new();

    if let Some(source) = vault.source.clone() {
        match source {
            Source::Bow { address } => messages.append(&mut swap_for_bow_deposit_messages(
                deps,
                &env,
                address,
                deposit,
                vault.slippage_tolerance,
            )?),
        };
    } else {
        update_vault(deps.storage, vault.id.into(), |stored_vault| {
            if let Some(mut stored_vault) = stored_vault {
                stored_vault.balance.amount += info.funds[0].amount;
                if !stored_vault.is_scheduled() && stored_vault.has_sufficient_funds() {
                    stored_vault.status = VaultStatus::Active
                }
                return Ok(stored_vault);
            }
            Err(StdError::NotFound {
                kind: format!("Vault {}", vault.id),
            })
        })?;
    }

    let vault_was_inactive = vault.is_inactive();

    let vault = get_vault(deps.storage, vault.id)?;

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

        messages.push(execute_trigger_message(env, vault_id));
    };

    Ok(Response::new()
        .add_attribute("method", "deposit")
        .add_messages(messages))
}
