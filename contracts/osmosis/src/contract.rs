#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use crate::error::ContractError;
use crate::handlers::get_pool::get_pool;
use crate::handlers::get_price::get_price;
use crate::handlers::lock_tokens::lock_tokens;
use crate::handlers::provide_liquidity::provide_liquidity;
use crate::handlers::swap::swap;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Swap { pool_id, denom_out } => swap(deps, env, info, pool_id, denom_out),
        ExecuteMsg::ProvideLiquidityAndLockTokens { pool_id } => {
            provide_liquidity(deps, env, info, pool_id)
        }
    }
}

pub const AFTER_PROVIDE_LIQUIDITY: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_PROVIDE_LIQUIDITY => lock_tokens(deps, env),
        _ => Ok(Response::default()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPool { pool_id } => to_binary(&get_pool(deps, pool_id)?),
        QueryMsg::GetPrice {
            pool_id,
            denom_in,
            amount_in,
            denom_out,
        } => to_binary(&get_price(
            deps, env, pool_id, denom_in, amount_in, denom_out,
        )?),
    }
}

#[cfg(test)]
mod tests {}
