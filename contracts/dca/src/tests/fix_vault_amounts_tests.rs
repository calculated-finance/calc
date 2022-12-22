use std::cmp::min;

use base::{
    events::event::{EventBuilder, EventData, ExecutionSkippedReason},
    helpers::math_helpers::checked_mul,
    triggers::trigger::TriggerConfiguration,
    vaults::vault::{PostExecutionAction, VaultStatus},
};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    BankMsg, Coin, Decimal, Event, Reply, SubMsg, SubMsgResponse, SubMsgResult, Timestamp, Uint128,
};
use fin_helpers::codes::ERROR_SWAP_SLIPPAGE_EXCEEDED;

use crate::{
    constants::TEN,
    contract::{AFTER_BANK_SWAP_REPLY_ID, AFTER_FIN_SWAP_REPLY_ID},
    handlers::{
        after_fin_swap::after_fin_swap, fix_vault_amounts::fix_vault_amounts,
        get_events_by_resource_id::get_events_by_resource_id,
    },
    state::{
        cache::{SwapCache, SWAP_CACHE},
        config::{create_custom_fee, get_config},
        triggers::get_trigger,
        vaults::get_vault,
    },
    tests::{
        helpers::{
            instantiate_contract, setup_active_vault_with_funds, setup_active_vault_with_low_funds,
            setup_active_vault_with_slippage_funds, setup_vault,
        },
        mocks::{ADMIN, DENOM_UKUJI},
    },
};

#[test]
fn should_adjust_swapped_amount_stat() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &vec![]));

    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());
    let receive_amount = Uint128::new(234312312);
    let updated_swapped_amount = Uint128::new(11000);
    let updated_receive_amount = Uint128::new(11000);

    SWAP_CACHE
        .save(
            deps.as_mut().storage,
            &SwapCache {
                swap_denom_balance: vault.balance.clone(),
                receive_denom_balance: Coin::new(0, vault.get_receive_denom()),
            },
        )
        .unwrap();

    deps.querier.update_balance(
        "cosmos2contract",
        vec![Coin::new(receive_amount.into(), vault.get_receive_denom())],
    );

    fix_vault_amounts(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        vault.id,
        Coin::new(updated_swapped_amount.into(), vault.get_swap_denom()),
        Coin::new(updated_receive_amount.into(), vault.get_receive_denom()),
    )
    .unwrap();

    let updated_vault = get_vault(&deps.storage, vault.id).unwrap();
    let config = get_config(&deps.storage).unwrap();

    let mut fee = config.swap_fee_percent * updated_receive_amount;

    vault
        .destinations
        .iter()
        .filter(|d| d.action == PostExecutionAction::ZDelegate)
        .for_each(|destination| {
            let allocation_amount =
                checked_mul(updated_receive_amount - fee, destination.allocation).unwrap();
            let allocation_automation_fee =
                checked_mul(allocation_amount, config.delegation_fee_percent).unwrap();
            fee = fee.checked_add(allocation_automation_fee).unwrap();
        });

    assert_eq!(updated_vault.swapped_amount.amount, updated_swapped_amount);
}

#[test]
fn should_adjust_received_amount_stat() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &vec![]));

    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());
    let receive_amount = Uint128::new(234312312);
    let updated_swapped_amount = Uint128::new(11000);
    let updated_receive_amount = Uint128::new(11000);

    SWAP_CACHE
        .save(
            deps.as_mut().storage,
            &SwapCache {
                swap_denom_balance: vault.balance.clone(),
                receive_denom_balance: Coin::new(0, vault.get_receive_denom()),
            },
        )
        .unwrap();

    deps.querier.update_balance(
        "cosmos2contract",
        vec![Coin::new(receive_amount.into(), vault.get_receive_denom())],
    );

    fix_vault_amounts(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        vault.id,
        Coin::new(updated_swapped_amount.into(), vault.get_swap_denom()),
        Coin::new(updated_receive_amount.into(), vault.get_receive_denom()),
    )
    .unwrap();

    let updated_vault = get_vault(&deps.storage, vault.id).unwrap();
    let config = get_config(&deps.storage).unwrap();

    let mut fee = config.swap_fee_percent * updated_receive_amount;

    vault
        .destinations
        .iter()
        .filter(|d| d.action == PostExecutionAction::ZDelegate)
        .for_each(|destination| {
            let allocation_amount =
                checked_mul(updated_receive_amount - fee, destination.allocation).unwrap();
            let allocation_automation_fee =
                checked_mul(allocation_amount, config.delegation_fee_percent).unwrap();
            fee = fee.checked_add(allocation_automation_fee).unwrap();
        });

    assert_eq!(
        updated_vault.received_amount.amount,
        updated_receive_amount - fee
    );
}

#[test]
fn should_disperse_funds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &vec![]));

    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());
    let receive_amount = Uint128::new(1000);
    let updated_swapped_amount = Uint128::new(11000);
    let updated_receive_amount = Uint128::new(11000);

    deps.querier.update_balance(
        "cosmos2contract",
        vec![Coin::new(receive_amount.into(), vault.get_receive_denom())],
    );

    let response = fix_vault_amounts(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        vault.id,
        Coin::new(updated_swapped_amount.into(), vault.get_swap_denom()),
        Coin::new(updated_receive_amount.into(), vault.get_receive_denom()),
    )
    .unwrap();

    let config = get_config(&deps.storage).unwrap();

    let automation_fee_rate = config
        .delegation_fee_percent
        .checked_mul(
            vault
                .destinations
                .iter()
                .filter(|destination| destination.action == PostExecutionAction::ZDelegate)
                .map(|destination| destination.allocation)
                .sum(),
        )
        .unwrap();

    let swap_fee = config.swap_fee_percent * updated_receive_amount;
    let total_after_swap_fee = updated_receive_amount - swap_fee;
    let automation_fee = checked_mul(total_after_swap_fee, automation_fee_rate).unwrap();
    let total_fee = swap_fee + automation_fee;
    let total_after_total_fee = updated_receive_amount - total_fee;

    assert!(response.messages.contains(&SubMsg::reply_on_success(
        BankMsg::Send {
            to_address: vault.destinations.first().unwrap().address.to_string(),
            amount: vec![Coin::new(
                total_after_total_fee.into(),
                vault.get_receive_denom(),
            )],
        },
        AFTER_BANK_SWAP_REPLY_ID,
    )));
}

#[test]
fn publishes_fix_amount_event() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &vec![]));
    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());
    let receive_amount = Uint128::new(1000);
    let updated_swapped_amount = Uint128::new(11000);
    let updated_receive_amount = Uint128::new(11000);

    deps.querier.update_balance(
        "cosmos2contract",
        vec![Coin::new(receive_amount.into(), vault.get_receive_denom())],
    );

    fix_vault_amounts(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        vault.id,
        Coin::new(updated_swapped_amount.into(), vault.get_swap_denom()),
        Coin::new(updated_receive_amount.into(), vault.get_receive_denom()),
    )
    .unwrap();

    let events = get_events_by_resource_id(deps.as_ref(), vault.id.clone(), None, None)
        .unwrap()
        .events;

    let config = get_config(&deps.storage).unwrap();

    let automation_fee_rate = config
        .delegation_fee_percent
        .checked_mul(
            vault
                .destinations
                .iter()
                .filter(|destination| destination.action == PostExecutionAction::ZDelegate)
                .map(|destination| destination.allocation)
                .sum(),
        )
        .unwrap();

    let swap_fee = config.swap_fee_percent * updated_receive_amount;
    let total_after_swap_fee = updated_receive_amount - swap_fee;
    let automation_fee = checked_mul(total_after_swap_fee, automation_fee_rate).unwrap();
    let total_fee = swap_fee + automation_fee;

    assert!(events.contains(
        &EventBuilder::new(
            vault.id,
            env.block.clone(),
            EventData::DcaFixVaultAmounts {
                expected_swapped_amount: Coin::new(updated_swapped_amount.into(), "base"),
                actual_swapped_amount: Coin::new(0, "base"),
                expected_received_amount: Coin::new(updated_receive_amount.into(), "quote"),
                actual_received_amount: Coin::new(0, "quote"),
                fee: Coin::new(total_fee.into(), "quote")
            }
        )
        .build(1)
    ));
}

#[test]
fn returns_fee_to_fee_collector() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &vec![]));
    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());
    let receive_amount = Uint128::new(1000);
    let updated_swapped_amount = Uint128::new(11000);
    let updated_receive_amount = Uint128::new(11000);

    deps.querier.update_balance(
        "cosmos2contract",
        vec![Coin::new(receive_amount.into(), vault.get_receive_denom())],
    );

    let response = fix_vault_amounts(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        vault.id,
        Coin::new(updated_swapped_amount.into(), vault.get_swap_denom()),
        Coin::new(updated_receive_amount.into(), vault.get_receive_denom()),
    )
    .unwrap();

    let config = get_config(&deps.storage).unwrap();

    let automation_fee_rate = config
        .delegation_fee_percent
        .checked_mul(
            vault
                .destinations
                .iter()
                .filter(|destination| destination.action == PostExecutionAction::ZDelegate)
                .map(|destination| destination.allocation)
                .sum(),
        )
        .unwrap();

    let swap_fee = config.swap_fee_percent * updated_receive_amount;
    let total_after_swap_fee = updated_receive_amount - swap_fee;
    let automation_fee = checked_mul(total_after_swap_fee, automation_fee_rate).unwrap();

    assert!(response.messages.contains(&SubMsg::new(BankMsg::Send {
        to_address: config.fee_collector.to_string(),
        amount: vec![Coin::new(swap_fee.into(), vault.get_receive_denom())]
    })));

    assert!(response.messages.contains(&SubMsg::new(BankMsg::Send {
        to_address: config.fee_collector.to_string(),
        amount: vec![Coin::new(automation_fee.into(), vault.get_receive_denom())]
    })));
}

// this tests read poorly
#[test]
fn with_correct_received_amount_should_do_nothing() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    instantiate_contract(deps.as_mut(), env.clone(), mock_info(ADMIN, &vec![]));
    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());
    let receive_amount = Uint128::new(11000);
    let updated_swapped_amount = Uint128::new(0);
    let updated_receive_amount = Uint128::new(0);

    assert_eq!(vault.received_amount.amount, Uint128::zero());

    deps.querier.update_balance(
        "cosmos2contract",
        vec![Coin::new(receive_amount.into(), vault.get_receive_denom())],
    );

    let response = fix_vault_amounts(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        vault.id,
        Coin::new(updated_swapped_amount.into(), vault.get_swap_denom()),
        Coin::new(updated_receive_amount.into(), vault.get_receive_denom()),
    )
    .unwrap();

    let events = get_events_by_resource_id(deps.as_ref(), vault.id.clone(), None, None)
        .unwrap()
        .events;

    assert!(events.is_empty());

    assert!(response.messages.is_empty())
}
