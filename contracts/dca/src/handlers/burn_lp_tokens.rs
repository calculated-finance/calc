use crate::{
    contract::AFTER_BURNING_LP_TOKENS,
    error::ContractError,
    state::{cache::BOW_CACHE, pools::get_pool},
};
use bow_helpers::msg::BowPoolsExecuteMsg;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, DepsMut, Env, Response, SubMsg, WasmMsg};

pub fn burn_lp_tokens(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut bow_cache = BOW_CACHE.load(deps.storage)?;

    let pool_config =
        get_pool(deps.storage, bow_cache.pool_address.clone())?.expect("Bow Pool config");

    bow_cache.withdrawal = pool_config
        .denoms
        .iter()
        .map(|denom| {
            deps.querier
                .query_balance(env.contract.address.to_string(), denom)
                .expect(format!("Balance for {}", denom).as_str())
        })
        .collect::<Vec<Coin>>();

    BOW_CACHE.save(deps.storage, &bow_cache)?;

    let lp_token_balance = deps
        .querier
        .query_balance(
            env.contract.address.to_string(),
            format!("factory/{}/ulp", bow_cache.pool_address),
        )
        .expect("LP token balance");

    let burn_lp_tokens_message = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bow_cache.pool_address.to_string(),
            msg: to_binary(&BowPoolsExecuteMsg::Withdraw {})?,
            funds: vec![lp_token_balance],
        }),
        AFTER_BURNING_LP_TOKENS,
    );

    Ok(Response::new().add_submessage(burn_lp_tokens_message))
}

#[cfg(test)]
mod burn_lp_tokens_tests {
    use crate::{
        contract::AFTER_BURNING_LP_TOKENS,
        handlers::create_bow_pool::create_bow_pool,
        state::cache::{BowCache, BOW_CACHE},
        tests::{helpers::instantiate_contract, mocks::ADMIN},
    };
    use bow_helpers::msg::BowPoolsExecuteMsg;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        to_binary, Addr, Coin, CosmosMsg, SubMsg, WasmMsg,
    };

    use super::burn_lp_tokens;

    #[test]
    pub fn sends_burn_tokens_message() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);
        let bow_pool_address = Addr::unchecked("bow-pool");

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        create_bow_pool(
            deps.as_mut(),
            info.clone(),
            bow_pool_address.clone(),
            vec!["base".to_string(), "quote".to_string()],
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
            .update_balance("cosmos2contract", vec![lp_token_balance.clone()]);

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

        let pool_denoms = vec!["base".to_string(), "quote".to_string()];

        create_bow_pool(
            deps.as_mut(),
            info.clone(),
            bow_pool_address.clone(),
            pool_denoms.clone(),
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
            .update_balance("cosmos2contract", vec![lp_token_balance.clone()]);

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
}
