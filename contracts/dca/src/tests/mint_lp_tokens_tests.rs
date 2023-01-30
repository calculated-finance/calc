use crate::{
    contract::AFTER_MINTING_LP_TOKENS,
    handlers::mint_lp_tokens::mint_lp_tokens,
    state::cache::{BowCache, SwapCache, BOW_CACHE, SWAP_CACHE},
    tests::{helpers::instantiate_contract, mocks::ADMIN},
};
use bow_helpers::msg::BowPoolsExecuteMsg;
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, Addr, Coin, CosmosMsg, SubMsg, WasmMsg,
};

#[test]
pub fn mints_tokens_when_deposit_is_full() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let bow_pool_address = Addr::unchecked("bow-pool");

    BOW_CACHE
        .save(
            deps.as_mut().storage,
            &BowCache {
                pool_address: bow_pool_address.clone(),
                deposit: vec![Coin::new(10000, "udemo")],
                withdrawal: vec![],
                lp_token_balance: None,
            },
        )
        .unwrap();

    SWAP_CACHE
        .save(
            deps.as_mut().storage,
            &SwapCache {
                receive_denom_balance: Coin::new(0, "ukuji"),
                swap_denom_balance: Coin::new(10000, "udemo"),
            },
        )
        .unwrap();

    deps.querier.update_balance(
        env.contract.address.clone(),
        vec![Coin::new(10000, "ukuji")],
    );

    let response = mint_lp_tokens(deps.as_mut(), &env).unwrap();

    assert!(response.messages.contains(&SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bow_pool_address.to_string(),
            msg: to_binary(&BowPoolsExecuteMsg::Deposit {}).unwrap(),
            funds: vec![Coin::new(10000, "udemo"), Coin::new(10000, "ukuji")],
        }),
        AFTER_MINTING_LP_TOKENS,
    )))
}

#[test]
pub fn does_not_mint_tokens_when_deposit_is_empty() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let bow_pool_address = Addr::unchecked("bow-pool");

    BOW_CACHE
        .save(
            deps.as_mut().storage,
            &BowCache {
                pool_address: bow_pool_address.clone(),
                deposit: vec![],
                withdrawal: vec![],
                lp_token_balance: None,
            },
        )
        .unwrap();

    SWAP_CACHE
        .save(
            deps.as_mut().storage,
            &SwapCache {
                receive_denom_balance: Coin::new(0, "ukuji"),
                swap_denom_balance: Coin::new(10000, "udemo"),
            },
        )
        .unwrap();

    deps.querier.update_balance(
        env.contract.address.clone(),
        vec![Coin::new(10000, "ukuji")],
    );

    let response = mint_lp_tokens(deps.as_mut(), &env).unwrap();

    assert!(response.messages.is_empty())
}

#[test]
pub fn updates_bow_cache_when_deposit_is_empty() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let bow_pool_address = Addr::unchecked("bow-pool");

    BOW_CACHE
        .save(
            deps.as_mut().storage,
            &BowCache {
                pool_address: bow_pool_address.clone(),
                deposit: vec![],
                withdrawal: vec![],
                lp_token_balance: None,
            },
        )
        .unwrap();

    SWAP_CACHE
        .save(
            deps.as_mut().storage,
            &SwapCache {
                receive_denom_balance: Coin::new(0, "ukuji"),
                swap_denom_balance: Coin::new(10000, "udemo"),
            },
        )
        .unwrap();

    deps.querier.update_balance(
        env.contract.address.clone(),
        vec![Coin::new(10000, "ukuji")],
    );

    mint_lp_tokens(deps.as_mut(), &env).unwrap();

    let bow_cache = BOW_CACHE.load(deps.as_ref().storage).unwrap();

    assert_eq!(bow_cache.deposit, vec![Coin::new(10000, "ukuji")])
}
