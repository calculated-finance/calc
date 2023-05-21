use crate::error::ContractError;
use crate::helpers::validation::{
    assert_sender_is_admin_or_vault_owner, assert_vault_is_not_cancelled,
};
use crate::state::disburse_escrow_tasks::save_disburse_escrow_task;
use crate::state::events::create_event;
<<<<<<< HEAD
use crate::state::old_triggers::delete_old_trigger;
use crate::state::old_vaults::{get_old_vault, update_old_vault};
use crate::types::event::{EventBuilder, EventData};
use base::triggers::trigger::OldTriggerConfiguration;
use base::vaults::vault::OldVaultStatus;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{BankMsg, DepsMut, Response, Uint128};
use cosmwasm_std::{Coin, CosmosMsg, Env, MessageInfo};
use fin_helpers::limit_orders::{create_retract_order_msg, create_withdraw_limit_order_msg};
use fin_helpers::queries::query_order_details;

pub fn cancel_vault(
=======
use crate::state::triggers::delete_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::types::event::{EventBuilder, EventData};
use crate::types::vault::{Vault, VaultStatus};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{BankMsg, DepsMut, Response, Uint128};
use cosmwasm_std::{Coin, Env, MessageInfo, SubMsg};

pub fn cancel_vault_handler(
>>>>>>> 1586f6e (add updated create handlers)
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
<<<<<<< HEAD
    let mut vault = get_old_vault(deps.storage, vault_id)?;

    assert_sender_is_admin_or_vault_owner(deps.storage, vault.owner.clone(), info.sender.clone())?;
=======
    let vault = get_vault(deps.storage, vault_id)?;

    assert_sender_is_admin_or_vault_owner(deps.storage, vault.owner.clone(), info.sender)?;
>>>>>>> 1586f6e (add updated create handlers)
    assert_vault_is_not_cancelled(&vault)?;

    create_event(
        deps.storage,
        EventBuilder::new(vault.id, env.block.clone(), EventData::DcaVaultCancelled {}),
    )?;

<<<<<<< HEAD
    if let Some(_) = vault.dca_plus_config {
=======
    if vault.escrowed_amount.amount > Uint128::zero() {
>>>>>>> 1586f6e (add updated create handlers)
        save_disburse_escrow_task(
            deps.storage,
            vault.id,
            vault.get_expected_execution_completed_date(env.block.time),
        )?;
    };

<<<<<<< HEAD
    let mut messages: Vec<CosmosMsg> = Vec::new();

    if vault.balance.amount > Uint128::zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
=======
    let mut submessages = Vec::<SubMsg>::new();

    if vault.balance.amount > Uint128::zero() {
        submessages.push(SubMsg::new(BankMsg::Send {
>>>>>>> 1586f6e (add updated create handlers)
            to_address: vault.owner.to_string(),
            amount: vec![vault.balance.clone()],
        }));
    }

<<<<<<< HEAD
    vault.status = OldVaultStatus::Cancelled;
    vault.balance = Coin::new(0, vault.get_swap_denom());

    update_old_vault(deps.storage, &vault)?;

    if let Some(trigger) = vault.trigger {
        match trigger {
            OldTriggerConfiguration::FinLimitOrder { order_idx, .. } => {
                if let Some(order_idx) = order_idx {
                    let limit_order =
                        query_order_details(deps.querier, vault.pair.address.clone(), order_idx)
                            .expect(&format!(
                                "Fin limit order exists at pair {}",
                                vault.pair.address.clone()
                            ));

                    if limit_order.offer_amount > Uint128::zero() {
                        messages.push(create_retract_order_msg(
                            vault.pair.address.clone(),
                            order_idx,
                        ));
                    }

                    if limit_order.filled_amount > Uint128::zero() {
                        messages.push(create_withdraw_limit_order_msg(
                            vault.pair.address.clone(),
                            order_idx,
                        ));
                    }
                }
            }
            _ => {}
        }

        delete_old_trigger(deps.storage, vault.id)?;
    }

    Ok(Response::new()
        .add_attribute("method", "cancel_vault")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id)
        .add_messages(messages))
=======
    let updated_vault = Vault {
        status: VaultStatus::Cancelled,
        balance: Coin::new(0, vault.get_swap_denom()),
        ..vault
    };

    update_vault(deps.storage, &updated_vault)?;

    delete_trigger(deps.storage, updated_vault.id)?;

    Ok(Response::new()
        .add_attribute("cancel_vault", "true")
        .add_attribute("vault_id", vault.id)
        .add_attribute("refunded_amount", vault.balance.to_string())
        .add_submessages(submessages))
}

#[cfg(test)]
mod cancel_vault_tests {
    use super::*;
    use crate::constants::ONE;
    use crate::handlers::get_events_by_resource_id::get_events_by_resource_id_handler;
    use crate::handlers::get_vault::get_vault_handler;
    use crate::state::disburse_escrow_tasks::get_disburse_escrow_tasks;
    use crate::tests::helpers::{instantiate_contract, setup_vault};
    use crate::tests::mocks::{ADMIN, DENOM_UKUJI};
    use crate::types::event::{EventBuilder, EventData};
    use crate::types::vault::{Vault, VaultStatus};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{BankMsg, SubMsg, Uint128};

    #[test]
    fn should_return_balance_to_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        let response = cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap();

        assert!(response.messages.contains(&SubMsg::new(BankMsg::Send {
            to_address: vault.owner.to_string(),
            amount: vec![vault.balance.clone()],
        })));
    }

    #[test]
    fn should_publish_vault_cancelled_event() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        cancel_vault_handler(deps.as_mut(), env.clone(), info, vault.id).unwrap();

        let events = get_events_by_resource_id_handler(deps.as_ref(), vault.id, None, None, None)
            .unwrap()
            .events;

        assert!(events.contains(
            &EventBuilder::new(vault.id, env.block, EventData::DcaVaultCancelled {}).build(1)
        ));
    }

    #[test]
    fn when_vault_has_time_trigger_should_cancel_vault() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        cancel_vault_handler(deps.as_mut(), env.clone(), info, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_eq!(vault.status, VaultStatus::Active);
        assert_eq!(updated_vault.status, VaultStatus::Cancelled);
    }

    #[test]
    fn should_empty_vault_balance() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        cancel_vault_handler(deps.as_mut(), env.clone(), info, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert!(vault.balance.amount.gt(&Uint128::zero()));
        assert!(updated_vault.balance.amount.is_zero());
    }

    #[test]
    fn on_already_cancelled_vault_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                status: VaultStatus::Cancelled,
                ..Vault::default()
            },
        );

        let err = cancel_vault_handler(deps.as_mut(), env.clone(), info, vault.id).unwrap_err();

        assert_eq!(err.to_string(), "Error: vault is already cancelled");
    }

    #[test]
    fn for_vault_with_different_owner_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        let err = cancel_vault_handler(
            deps.as_mut(),
            env.clone(),
            mock_info("not-the-owner", &[]),
            vault.id,
        )
        .unwrap_err();

        assert_eq!(err.to_string(), "Unauthorized");
    }

    #[test]
    fn should_delete_the_trigger() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        cancel_vault_handler(deps.as_mut(), env.clone(), info, vault.id).unwrap();

        let updated_vault = get_vault_handler(deps.as_ref(), vault.id).unwrap().vault;

        assert_ne!(vault.trigger, None);
        assert_eq!(updated_vault.trigger, None);
    }

    #[test]
    fn with_escrowed_balance_should_save_disburse_escrow_task() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                escrowed_amount: Coin::new(ONE.into(), DENOM_UKUJI.to_string()),
                ..Vault::default()
            },
        );

        cancel_vault_handler(deps.as_mut(), env.clone(), info, vault.id).unwrap();

        let disburse_escrow_tasks_before = get_disburse_escrow_tasks(
            deps.as_ref().storage,
            vault
                .get_expected_execution_completed_date(env.block.time)
                .minus_seconds(10),
            Some(100),
        )
        .unwrap();

        assert!(disburse_escrow_tasks_before.is_empty());

        let disburse_escrow_tasks_after = get_disburse_escrow_tasks(
            deps.as_ref().storage,
            vault
                .get_expected_execution_completed_date(env.block.time)
                .plus_seconds(10),
            Some(100),
        )
        .unwrap();

        assert!(disburse_escrow_tasks_after.contains(&vault.id));
    }
>>>>>>> 1586f6e (add updated create handlers)
}
