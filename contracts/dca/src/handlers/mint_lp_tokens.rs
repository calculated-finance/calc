use crate::{
    contract::AFTER_MINTING_LP_TOKENS,
    error::ContractError,
    state::cache::{BOW_CACHE, SWAP_CACHE},
};
use bow_helpers::msg::BowPoolsExecuteMsg;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, DepsMut, Env, Response, SubMsg, WasmMsg};

pub fn mint_lp_tokens(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut bow_cache = BOW_CACHE.load(deps.storage)?;
    let swap_cache = SWAP_CACHE.load(deps.storage)?;

    let receive_denom_balance = &deps.querier.query_balance(
        &env.contract.address,
        &swap_cache.receive_denom_balance.denom,
    )?;

    let coin_received = Coin::new(
        (receive_denom_balance.amount - swap_cache.receive_denom_balance.amount).into(),
        receive_denom_balance.denom.clone(),
    );

    bow_cache.deposit.push(coin_received);
    BOW_CACHE.save(deps.storage, &bow_cache)?;

    let mut response = Response::new();

    if bow_cache.deposit.len() == 2 {
        response = response.add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: bow_cache.pool_address.to_string(),
                msg: to_binary(&BowPoolsExecuteMsg::Deposit {}).unwrap(),
                funds: bow_cache.deposit,
            }),
            AFTER_MINTING_LP_TOKENS,
        ));
    }

    Ok(response)
}
