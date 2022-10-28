use std::str::FromStr;

use super::mocks::fin_contract_fail_slippage_tolerance;
use crate::constants::{ONE, ONE_HUNDRED, ONE_THOUSAND, TEN};
use crate::msg::{ExecuteMsg, QueryMsg, TriggerIdsResponse, VaultResponse};
use crate::tests::helpers::{
    assert_address_balances, assert_events_published, assert_vault_balance,
};
use crate::tests::mocks::{
    fin_contract_filled_limit_order, fin_contract_partially_filled_order,
    fin_contract_pass_slippage_tolerance, fin_contract_unfilled_limit_order, MockApp, ADMIN,
    DENOM_UKUJI, DENOM_UTEST, USER,
};
use base::events::event::{EventBuilder, EventData};
use base::helpers::math_helpers::checked_mul;
use base::vaults::vault::{Destination, PositionType, PostExecutionAction, VaultStatus};
use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, Uint128};
use cw_multi_test::Executor;

#[test]
fn for_filled_fin_limit_order_trigger_should_update_address_balances() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "fin",
        );

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit - swap_amount,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (
                &mock.fin_contract_address,
                DENOM_UTEST,
                ONE_THOUSAND + swap_amount,
            ),
        ],
    );

    let vault_id = mock.vault_ids.get("fin").unwrap().to_owned();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id,
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, Uint128::new(0)),
            (&user_address, DENOM_UTEST, swap_amount_after_fee),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit - swap_amount,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );
}

#[test]
fn for_filled_fin_limit_order_trigger_should_update_vault_balance() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "fin",
        );

    let vault_id = mock.vault_ids.get("fin").unwrap().to_owned();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id,
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        Uint128::new(1),
        vault_deposit - swap_amount,
    );
}

#[test]
fn for_filled_fin_limit_order_trigger_should_update_vault_stats() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "fin",
        );

    let vault_id = mock.vault_ids.get("fin").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_id,
            },
            &[],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id,
                address: user_address.clone(),
            },
        )
        .unwrap();

    assert_eq!(vault_response.vault.swapped_amount, swap_amount);
    assert_eq!(
        vault_response.vault.received_amount,
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap()
    );
}

#[test]
fn for_filled_fin_limit_order_trigger_should_publish_events() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "fin",
        );

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    let vault_id = mock.vault_ids.get("fin").unwrap().to_owned();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id,
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    assert_events_published(
        &mock,
        vault_id,
        &[
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionTriggered {
                    base_denom: DENOM_UTEST.to_string(),
                    quote_denom: DENOM_UKUJI.to_string(),
                    position_type: PositionType::Enter,
                    asset_price: Decimal256::from_str("1.0").unwrap(),
                },
            )
            .build(3),
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionCompleted {
                    sent: Coin::new(swap_amount.into(), DENOM_UKUJI),
                    received: Coin::new(swap_amount.into(), DENOM_UTEST),
                    fee: Coin::new((swap_amount - swap_amount_after_fee).into(), DENOM_UTEST),
                },
            )
            .build(4),
        ],
    );
}

#[test]
fn for_filled_fin_limit_order_trigger_should_delete_existing_fin_limit_order_trigger() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "fin",
        );

    let vault_id = mock.vault_ids.get("fin").unwrap().to_owned();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id,
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        response.root_cause().to_string(),
        "Error: trigger execution time has not yet elapsed"
    )
}

#[test]
fn for_filled_fin_limit_order_trigger_should_create_new_time_trigger() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "fin",
        );

    let vault_id = mock.vault_ids.get("fin").unwrap().to_owned();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id,
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    mock.elapse_time(3700);

    let get_time_trigger_ids_response: TriggerIdsResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address.clone(),
            &QueryMsg::GetTimeTriggerIds {},
        )
        .unwrap();

    assert_eq!(get_time_trigger_ids_response.trigger_ids.len(), 1);
}

#[test]
fn for_filled_fin_limit_order_trigger_should_distribute_to_multiple_destinations_properly() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut destinations = vec![];

    for i in 0..5 {
        destinations.push(Destination {
            address: Addr::unchecked(format!("{}-{:?}", USER, i)),
            allocation: Decimal::percent(20),
            action: PostExecutionAction::Send,
        });
    }

    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            Some(destinations.clone()),
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "fin",
        );

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    assert_address_balances(
        &mock,
        &destinations
            .iter()
            .map(|destination| {
                (
                    &destination.address,
                    DENOM_UTEST,
                    swap_amount_after_fee * destination.allocation,
                )
            })
            .collect::<Vec<_>>(),
    );
}

#[test]
fn for_partially_filled_limit_order_should_return_error() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_partially_filled_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_partially_filled_fin_limit_price_trigger(
            &user_address,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("fin").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        response.root_cause().to_string(),
        "Error: fin limit order has not been completely filled"
    );
}

#[test]
fn for_partially_filled_limit_order_should_not_change_address_balances() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_partially_filled_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_partially_filled_fin_limit_price_trigger(
            &user_address,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, Uint128::new(0)),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit - swap_amount,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (
                &mock.fin_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + swap_amount / Uint128::new(2),
            ),
            (
                &mock.fin_contract_address,
                DENOM_UTEST,
                ONE_THOUSAND + swap_amount / Uint128::new(2),
            ),
        ],
    );

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("fin").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap_err();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, Uint128::new(0)),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit - swap_amount,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (
                &mock.fin_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + swap_amount / Uint128::new(2),
            ),
            (
                &mock.fin_contract_address,
                DENOM_UTEST,
                ONE_THOUSAND + swap_amount / Uint128::new(2),
            ),
        ],
    );
}

#[test]
fn for_partially_filled_limit_order_should_not_change_vault_balance() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_partially_filled_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_partially_filled_fin_limit_price_trigger(
            &user_address,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("fin").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap_err();

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        Uint128::new(1),
        vault_deposit,
    );
}

#[test]
fn for_ready_time_trigger_should_update_addess_balances() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, Uint128::new(0)),
            (&user_address, DENOM_UTEST, swap_amount_after_fee),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit - swap_amount,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (
                &mock.fin_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + swap_amount,
            ),
            (
                &mock.fin_contract_address,
                DENOM_UTEST,
                ONE_THOUSAND - swap_amount,
            ),
        ],
    );
}

#[test]
fn for_ready_time_trigger_should_update_vault_balance() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        Uint128::new(1),
        TEN - ONE,
    );
}

#[test]
fn for_ready_time_trigger_should_update_vault_stats() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("time").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    assert_eq!(vault_response.vault.swapped_amount, swap_amount);
    assert_eq!(
        vault_response.vault.received_amount,
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap()
    );
}

#[test]
fn for_ready_time_trigger_should_create_events() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    let vault_id = mock.vault_ids.get("time").unwrap().to_owned();

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    assert_events_published(
        &mock,
        vault_id,
        &[
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionTriggered {
                    base_denom: DENOM_UTEST.to_string(),
                    quote_denom: DENOM_UKUJI.to_string(),
                    position_type: PositionType::Enter,
                    asset_price: Decimal256::from_str("1.0").unwrap(),
                },
            )
            .build(3),
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionCompleted {
                    sent: Coin::new(swap_amount.into(), DENOM_UKUJI),
                    received: Coin::new(swap_amount.into(), DENOM_UTEST),
                    fee: Coin::new((swap_amount - swap_amount_after_fee).into(), DENOM_UTEST),
                },
            )
            .build(4),
        ],
    );
}

#[test]
fn for_ready_time_trigger_should_delete_current_time_trigger() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        response.root_cause().to_string(),
        "Error: trigger execution time has not yet elapsed"
    )
}

#[test]
fn for_ready_time_trigger_should_create_new_time_trigger() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_pass_slippage_tolerance())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    mock.elapse_time(3700);

    let get_time_trigger_ids_response: TriggerIdsResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address.clone(),
            &QueryMsg::GetTimeTriggerIds {},
        )
        .unwrap();

    assert_eq!(get_time_trigger_ids_response.trigger_ids.len(), 1);
}

#[test]
fn for_ready_time_trigger_should_distribute_to_multiple_destinations_properly() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut destinations = vec![];

    for i in 0..5 {
        destinations.push(Destination {
            address: Addr::unchecked(format!("{}-{:?}", USER, i)),
            allocation: Decimal::percent(20),
            action: PostExecutionAction::Send,
        });
    }

    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            Some(destinations.clone()),
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    assert_address_balances(
        &mock,
        &destinations
            .iter()
            .map(|destination| {
                (
                    &destination.address,
                    DENOM_UTEST,
                    swap_amount_after_fee * destination.allocation,
                )
            })
            .collect::<Vec<_>>(),
    );
}

#[test]
fn for_ready_time_trigger_within_price_threshold_should_succeed() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            Some(Decimal256::from_str("1.5").unwrap()),
        );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    let vault_id = mock.vault_ids.get("time").unwrap().to_owned();

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    assert_events_published(
        &mock,
        vault_id,
        &[
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionTriggered {
                    base_denom: DENOM_UTEST.to_string(),
                    quote_denom: DENOM_UKUJI.to_string(),
                    position_type: PositionType::Enter,
                    asset_price: Decimal256::from_str("1.0").unwrap(),
                },
            )
            .build(3),
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionCompleted {
                    sent: Coin::new(swap_amount.into(), DENOM_UKUJI),
                    received: Coin::new(swap_amount.into(), DENOM_UTEST),
                    fee: Coin::new((swap_amount - swap_amount_after_fee).into(), DENOM_UTEST),
                },
            )
            .build(4),
        ],
    );
}

#[test]
fn for_ready_time_trigger_outside_of_price_threshold_should_skip_execution() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;

    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, user_balance, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            Some(Decimal256::from_str("0.9").unwrap()),
        );

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );

    let vault_id = mock.vault_ids.get("time").unwrap().to_owned();

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );

    assert_events_published(
        &mock,
        vault_id,
        &[
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionTriggered {
                    base_denom: DENOM_UTEST.to_string(),
                    quote_denom: DENOM_UKUJI.to_string(),
                    position_type: PositionType::Enter,
                    asset_price: Decimal256::from_str("1.0").unwrap(),
                },
            )
            .build(3),
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionSkipped {
                    reason: base::events::event::ExecutionSkippedReason::PriceThresholdExceeded {
                        price: Decimal256::from_str("1").unwrap(),
                    },
                },
            )
            .build(4),
        ],
    );

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        Uint128::new(1),
        TEN,
    );
}

#[test]
fn for_ready_time_trigger_when_slippage_exceeds_limit_should_skip_execution() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_fail_slippage_tolerance())
        .with_funds_for(&user_address, TEN, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );

    mock.elapse_time(10);

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        Uint128::new(1),
        vault_deposit,
    );
}

#[test]
fn for_not_ready_time_trigger_should_fail() {
    let user_address = Addr::unchecked(USER);
    let user_balance = TEN;
    let vault_deposit = TEN;
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_unfilled_limit_order())
        .with_funds_for(&user_address, TEN, DENOM_UKUJI)
        .with_vault_with_time_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI),
            swap_amount,
            "time",
            None,
        );

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );

    let response = mock
        .app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        response.root_cause().to_string(),
        "Error: trigger execution time has not yet elapsed"
    );

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_balance - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::new(0)),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UTEST, ONE_THOUSAND),
        ],
    );

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        Uint128::new(1),
        TEN,
    );
}

#[test]
fn until_vault_is_empty_should_update_address_balances() {
    let user_address = Addr::unchecked(USER);
    let user_funds = ONE_HUNDRED;
    let vault_deposit = ONE * Uint128::new(3) / Uint128::new(2);
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_funds, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    let vault_deposit_after_fee =
        vault_deposit - checked_mul(vault_deposit, mock.fee_percent).ok().unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, user_funds - vault_deposit),
            (&user_address, DENOM_UTEST, Uint128::zero()),
            (
                &mock.dca_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + vault_deposit - swap_amount,
            ),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (&mock.fin_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (
                &mock.fin_contract_address,
                DENOM_UTEST,
                ONE_THOUSAND + swap_amount,
            ),
        ],
    );

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("fin").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    mock.elapse_time(3700);

    let time_triggers: TriggerIdsResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetTimeTriggerIds {})
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: time_triggers.trigger_ids[0],
            },
            &[],
        )
        .unwrap();

    assert_address_balances(
        &mock,
        &[
            (&user_address, DENOM_UKUJI, ONE_HUNDRED - vault_deposit),
            (&user_address, DENOM_UTEST, vault_deposit_after_fee),
            (&mock.dca_contract_address, DENOM_UKUJI, ONE_THOUSAND),
            (&mock.dca_contract_address, DENOM_UTEST, ONE_THOUSAND),
            (
                &mock.fin_contract_address,
                DENOM_UKUJI,
                ONE_THOUSAND + swap_amount / Uint128::new(2),
            ),
            (
                &mock.fin_contract_address,
                DENOM_UTEST,
                ONE_THOUSAND - swap_amount / Uint128::new(2),
            ),
        ],
    );
}

#[test]
fn until_vault_is_empty_should_update_vault_balance() {
    let user_address = Addr::unchecked(USER);
    let user_funds = ONE_HUNDRED;
    let vault_deposit = ONE * Uint128::new(3) / Uint128::new(2);
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_funds, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("fin").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    mock.elapse_time(3700);

    let time_triggers: TriggerIdsResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetTimeTriggerIds {})
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: time_triggers.trigger_ids[0],
            },
            &[],
        )
        .unwrap();

    assert_vault_balance(
        &mock,
        &mock.dca_contract_address,
        user_address,
        vault_response.vault.id,
        Uint128::new(0),
    );
}

#[test]
fn until_vault_is_empty_should_update_vault_status() {
    let user_address = Addr::unchecked(USER);
    let user_funds = ONE_HUNDRED;
    let vault_deposit = ONE * Uint128::new(3) / Uint128::new(2);
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_funds, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("fin").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    mock.elapse_time(3700);

    let time_triggers: TriggerIdsResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetTimeTriggerIds {})
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: time_triggers.trigger_ids[0],
            },
            &[],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("time").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    assert_eq!(vault_response.vault.status, VaultStatus::Inactive);
}

#[test]
fn until_vault_is_empty_should_update_vault_stats() {
    let user_address = Addr::unchecked(USER);
    let user_funds = ONE_HUNDRED;
    let vault_deposit = ONE * Uint128::new(3) / Uint128::new(2);
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_funds, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    let vault_deposit_after_fee =
        vault_deposit - checked_mul(vault_deposit, mock.fee_percent).ok().unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("fin").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_response.vault.id,
            },
            &[],
        )
        .unwrap();

    mock.elapse_time(3700);

    let time_triggers: TriggerIdsResponse = mock
        .app
        .wrap()
        .query_wasm_smart(&mock.dca_contract_address, &QueryMsg::GetTimeTriggerIds {})
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: time_triggers.trigger_ids[0],
            },
            &[],
        )
        .unwrap();

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id: mock.vault_ids.get("time").unwrap().to_owned(),
                address: user_address.clone(),
            },
        )
        .unwrap();

    assert_eq!(vault_response.vault.swapped_amount, vault_deposit);
    assert_eq!(
        vault_response.vault.received_amount,
        vault_deposit_after_fee
    );
}

#[test]
fn until_vault_is_empty_should_create_events() {
    let user_address = Addr::unchecked(USER);
    let user_funds = ONE_HUNDRED;
    let vault_deposit = ONE * Uint128::new(3) / Uint128::new(2);
    let swap_amount = ONE;
    let mut mock = MockApp::new(fin_contract_filled_limit_order())
        .with_funds_for(&user_address, user_funds, DENOM_UKUJI)
        .with_vault_with_filled_fin_limit_price_trigger(
            &user_address,
            None,
            Coin::new(vault_deposit.into(), DENOM_UKUJI.to_string()),
            swap_amount,
            "fin",
        );

    let vault_id = mock.vault_ids.get("fin").unwrap().to_owned();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_id,
            },
            &[],
        )
        .unwrap();

    let initial_block_info = mock.app.block_info();

    mock.elapse_time(3700);

    let vault_response: VaultResponse = mock
        .app
        .wrap()
        .query_wasm_smart(
            &mock.dca_contract_address,
            &&QueryMsg::GetVault {
                vault_id,
                address: user_address.clone(),
            },
        )
        .unwrap();

    mock.app
        .execute_contract(
            Addr::unchecked(ADMIN),
            mock.dca_contract_address.clone(),
            &ExecuteMsg::ExecuteTrigger {
                trigger_id: vault_id,
            },
            &[],
        )
        .unwrap();

    let swap_amount_after_fee =
        swap_amount - checked_mul(swap_amount, mock.fee_percent).ok().unwrap();

    let remaining_swap_amount = vault_response.vault.balance.amount;
    let remaining_swap_amount_after_fee = remaining_swap_amount
        - checked_mul(remaining_swap_amount, mock.fee_percent)
            .ok()
            .unwrap();

    assert_events_published(
        &mock,
        vault_id,
        &[
            EventBuilder::new(
                vault_id,
                initial_block_info.clone(),
                EventData::DCAVaultExecutionTriggered {
                    base_denom: DENOM_UTEST.to_string(),
                    quote_denom: DENOM_UKUJI.to_string(),
                    position_type: PositionType::Enter,
                    asset_price: Decimal256::from_str("1.0").unwrap(),
                },
            )
            .build(3),
            EventBuilder::new(
                vault_id,
                initial_block_info.clone(),
                EventData::DCAVaultExecutionCompleted {
                    sent: Coin::new(swap_amount.into(), DENOM_UKUJI),
                    received: Coin::new(swap_amount.into(), DENOM_UTEST),
                    fee: Coin::new((swap_amount - swap_amount_after_fee).into(), DENOM_UTEST),
                },
            )
            .build(4),
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionTriggered {
                    base_denom: DENOM_UTEST.to_string(),
                    quote_denom: DENOM_UKUJI.to_string(),
                    position_type: PositionType::Enter,
                    asset_price: Decimal256::from_str("1.0").unwrap(),
                },
            )
            .build(5),
            EventBuilder::new(
                vault_id,
                mock.app.block_info(),
                EventData::DCAVaultExecutionCompleted {
                    sent: Coin::new(remaining_swap_amount.into(), DENOM_UKUJI),
                    received: Coin::new(remaining_swap_amount.into(), DENOM_UTEST),
                    fee: Coin::new(
                        (remaining_swap_amount - remaining_swap_amount_after_fee).into(),
                        DENOM_UTEST,
                    ),
                },
            )
            .build(6),
        ],
    );
}
