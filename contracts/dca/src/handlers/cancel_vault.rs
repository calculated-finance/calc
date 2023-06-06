use crate::error::ContractError;
use crate::helpers::validation::{
    assert_sender_is_admin_or_vault_owner, assert_vault_is_not_cancelled,
};
use crate::state::disburse_escrow_tasks::save_disburse_escrow_task;
use crate::state::events::create_event;
use crate::state::pairs::find_pair;
use crate::state::triggers::delete_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::types::event::{EventBuilder, EventData};
use crate::types::trigger::TriggerConfiguration;
use crate::types::vault::{Vault, VaultStatus};
use cosmwasm_std::{to_binary, BankMsg, DepsMut, Response, Uint128, WasmMsg};
use cosmwasm_std::{Env, MessageInfo, SubMsg};
use exchange::msg::ExecuteMsg;
use shared::coin::empty_of;

pub fn cancel_vault_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    let vault = get_vault(deps.storage, vault_id)?;

    assert_sender_is_admin_or_vault_owner(deps.storage, vault.owner.clone(), info.sender)?;
    assert_vault_is_not_cancelled(&vault)?;

    create_event(
        deps.storage,
        EventBuilder::new(vault.id, env.block.clone(), EventData::DcaVaultCancelled {}),
    )?;

    if vault.escrowed_amount.amount > Uint128::zero() {
        save_disburse_escrow_task(
            deps.storage,
            vault.id,
            vault.get_expected_execution_completed_date(env.block.time),
        )?;
    };

    let mut submessages = Vec::<SubMsg>::new();

    if vault.balance.amount > Uint128::zero() {
        submessages.push(SubMsg::new(BankMsg::Send {
            to_address: vault.owner.to_string(),
            amount: vec![vault.balance.clone()],
        }));
    }

    let updated_vault = update_vault(
        deps.storage,
        Vault {
            status: VaultStatus::Cancelled,
            balance: empty_of(vault.balance.clone()),
            ..vault.clone()
        },
    )?;

    if let Some(TriggerConfiguration::Price { order_idx, .. }) = vault.trigger {
        if let Some(order_idx) = order_idx {
            let pair = find_pair(deps.storage, updated_vault.denoms()).unwrap();

            submessages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: pair.address.to_string(),
                msg: to_binary(&ExecuteMsg::WithdrawOrder {
                    order_idx,
                    denoms: vault.denoms(),
                })
                .unwrap(),
                funds: vec![],
            }));

            submessages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: pair.address.to_string(),
                msg: to_binary(&ExecuteMsg::RetractOrder {
                    order_idx,
                    denoms: vault.denoms(),
                })
                .unwrap(),
                funds: vec![],
            }));
        };
    };

    delete_trigger(deps.storage, vault.id)?;

    Ok(Response::new()
        .add_attribute("cancel_vault", "true")
        .add_attribute("vault_id", vault.id)
        .add_attribute("owner", vault.owner)
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
    use crate::tests::mocks::{calc_mock_dependencies, ADMIN, DENOM_UKUJI};
    use crate::types::event::{EventBuilder, EventData};
    use crate::types::vault::{Vault, VaultStatus};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{BankMsg, Coin, Decimal, SubMsg, Uint128};

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
            amount: vec![vault.balance],
        })));
    }

    #[test]
    fn with_price_trigger_should_return_balance_to_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                trigger: Some(TriggerConfiguration::Price {
                    target_price: Decimal::percent(200),
                    order_idx: Some(Uint128::new(28)),
                }),
                ..Vault::default()
            },
        );

        let response = cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap();

        assert!(response.messages.contains(&SubMsg::new(BankMsg::Send {
            to_address: vault.owner.to_string(),
            amount: vec![vault.balance],
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

        cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap();

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

        cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap();

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

        let err = cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap_err();

        assert_eq!(err.to_string(), "Error: vault is already cancelled");
    }

    #[test]
    fn for_vault_with_different_owner_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info);

        let vault = setup_vault(deps.as_mut(), env.clone(), Vault::default());

        let err = cancel_vault_handler(
            deps.as_mut(),
            env,
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

        cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap();

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

    #[test]
    fn should_retract_limit_order() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let order_idx = Uint128::new(123);

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                trigger: Some(TriggerConfiguration::Price {
                    target_price: Decimal::percent(200),
                    order_idx: Some(order_idx),
                }),
                ..Vault::default()
            },
        );

        let response = cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap();

        let pair = find_pair(deps.as_ref().storage, vault.denoms()).unwrap();

        assert_eq!(
            response.messages.get(2).unwrap(),
            &SubMsg::new(WasmMsg::Execute {
                contract_addr: pair.address.to_string(),
                msg: to_binary(&ExecuteMsg::RetractOrder {
                    order_idx,
                    denoms: vault.denoms()
                })
                .unwrap(),
                funds: vec![]
            })
        );
    }

    #[test]
    fn should_withdraw_limit_order() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let order_idx = Uint128::new(123);

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Vault {
                trigger: Some(TriggerConfiguration::Price {
                    target_price: Decimal::percent(200),
                    order_idx: Some(order_idx),
                }),
                ..Vault::default()
            },
        );

        let response = cancel_vault_handler(deps.as_mut(), env, info, vault.id).unwrap();

        let pair = find_pair(deps.as_ref().storage, vault.denoms()).unwrap();

        assert_eq!(
            response.messages.get(1).unwrap(),
            &SubMsg::new(WasmMsg::Execute {
                contract_addr: pair.address.to_string(),
                msg: to_binary(&ExecuteMsg::WithdrawOrder {
                    order_idx,
                    denoms: vault.denoms()
                })
                .unwrap(),
                funds: vec![]
            })
        );
    }
}
