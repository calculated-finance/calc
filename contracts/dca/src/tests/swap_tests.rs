use crate::{
    handlers::{create_pair::create_pair, swap::swap},
    state::cache::SWAP_CACHE,
    tests::{helpers::instantiate_contract, mocks::ADMIN},
    types::reply_config::ReplyConfig,
};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, Addr, Coin, CosmosMsg, ReplyOn, SubMsg, Uint128, WasmMsg,
};
use kujira::fin::ExecuteMsg;

#[test]
fn sends_swap_message_without_reply() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let swap_amount = Coin::new(100000, "base");
    let info = mock_info(&env.contract.address.to_string(), &[swap_amount.clone()]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let pair_address = Addr::unchecked("pair");

    create_pair(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &[]),
        pair_address.clone(),
        "base".to_string(),
        "quote".to_string(),
    )
    .unwrap();

    let response = swap(deps.as_mut(), &env, &info, pair_address.clone(), None, None).unwrap();

    assert!(response
        .messages
        .contains(&SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_address.to_string(),
            funds: vec![swap_amount],
            msg: to_binary(&ExecuteMsg::Swap {
                offer_asset: None,
                belief_price: None,
                max_spread: None,
                to: None
            })
            .unwrap()
        }))))
}

#[test]
fn sends_swap_message_with_reply() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let swap_amount = Coin::new(100000, "base");
    let info = mock_info(&env.contract.address.to_string(), &[swap_amount.clone()]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let pair_address = Addr::unchecked("pair");

    create_pair(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &[]),
        pair_address.clone(),
        "base".to_string(),
        "quote".to_string(),
    )
    .unwrap();

    let reply_id = 1;

    let response = swap(
        deps.as_mut(),
        &env,
        &info,
        pair_address.clone(),
        None,
        Some(ReplyConfig {
            id: reply_id,
            on: ReplyOn::Success,
        }),
    )
    .unwrap();

    assert!(response.messages.contains(&SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_address.to_string(),
            funds: vec![swap_amount],
            msg: to_binary(&ExecuteMsg::Swap {
                offer_asset: None,
                belief_price: None,
                max_spread: None,
                to: None
            })
            .unwrap()
        }),
        reply_id
    )))
}

#[test]
fn updates_swap_cache() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let swap_amount = Coin::new(100000, "base");
    let info = mock_info(&env.contract.address.to_string(), &[swap_amount.clone()]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let pair_address = Addr::unchecked("pair");

    create_pair(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &[]),
        pair_address.clone(),
        "base".to_string(),
        "quote".to_string(),
    )
    .unwrap();

    let reply_id = 1;

    swap(
        deps.as_mut(),
        &env,
        &info,
        pair_address.clone(),
        None,
        Some(ReplyConfig {
            id: reply_id,
            on: ReplyOn::Success,
        }),
    )
    .unwrap();

    let swap_cache = SWAP_CACHE.load(&deps.storage).unwrap();

    assert_eq!(swap_cache.swap_denom_balance.denom, swap_amount.denom);
    assert_eq!(swap_cache.swap_denom_balance.amount, Uint128::zero());
}
