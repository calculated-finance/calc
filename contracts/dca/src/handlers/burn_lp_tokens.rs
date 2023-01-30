use crate::{
    contract::AFTER_BURNING_LP_TOKENS,
    error::ContractError,
    state::{cache::BOW_CACHE, pools::get_pool},
};
use bow_helpers::msg::BowPoolsExecuteMsg;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, DepsMut, Env, Response, SubMsg, WasmMsg};

pub fn burn_lp_tokens(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut bow_cache = BOW_CACHE.load(deps.storage)?;

    let pool_config = get_pool(deps.storage, &bow_cache.pool_address)?.expect("Bow Pool config");

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
