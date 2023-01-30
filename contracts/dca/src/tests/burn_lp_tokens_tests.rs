use crate::{
    contract::AFTER_BURNING_LP_TOKENS,
    handlers::{burn_lp_tokens::burn_lp_tokens, create_bow_pool::create_bow_pool},
    state::cache::{BowCache, BOW_CACHE},
    tests::{helpers::instantiate_contract, mocks::ADMIN},
};
use bow_helpers::msg::BowPoolsExecuteMsg;
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, Addr, Coin, CosmosMsg, SubMsg, WasmMsg,
};

#[test]
pub fn sends_burn_tokens_message() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);
    let bow_pool_address = Addr::unchecked("bow-pool");

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    create_bow_pool(
        deps.as_mut(),
        &info,
        &bow_pool_address,
        ["base".to_string(), "quote".to_string()],
    )
    .unwrap();

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

    let lp_token_balance = Coin::new(100000, format!("factory/{}/ulp", bow_pool_address));

    deps.querier
        .update_balance(env.contract.address.clone(), vec![lp_token_balance.clone()]);

    let response = burn_lp_tokens(deps.as_mut(), env).unwrap();

    assert!(response.messages.contains(&SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bow_pool_address.to_string(),
            msg: to_binary(&BowPoolsExecuteMsg::Withdraw {}).unwrap(),
            funds: vec![lp_token_balance],
        }),
        AFTER_BURNING_LP_TOKENS,
    )))
}

#[test]
pub fn adds_initial_pool_denom_balances_to_bow_cache() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &[]);
    let bow_pool_address = Addr::unchecked("bow-pool");

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let pool_denoms = ["base".to_string(), "quote".to_string()];

    create_bow_pool(deps.as_mut(), &info, &bow_pool_address, pool_denoms.clone()).unwrap();

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

    let lp_token_balance = Coin::new(100000, format!("factory/{}/ulp", bow_pool_address));

    deps.querier
        .update_balance(env.contract.address.clone(), vec![lp_token_balance.clone()]);

    burn_lp_tokens(deps.as_mut(), env).unwrap();

    let pre_withdrawal_balances = BOW_CACHE.load(&deps.storage).unwrap().withdrawal;

    assert_eq!(
        pre_withdrawal_balances,
        pool_denoms
            .iter()
            .map(|denom| Coin::new(0, denom))
            .collect::<Vec<Coin>>()
    )
}
