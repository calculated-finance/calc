#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use shared::cw20::from_cw20_receive_msg;

use crate::error::ContractError;
use crate::handlers::get_expected_receive_amount::get_expected_receive_amount_handler;
use crate::handlers::get_twap_to_now::get_twap_to_now_handler;
use crate::handlers::swap::{return_swapped_funds, swap_handler};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::msg::{InstantiateMsg, MigrateMsg};
use crate::state::config::{get_config, update_config};
use crate::types::config::Config;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:astrovault_calc";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(msg.admin.as_str())?;
    deps.api.addr_validate(msg.router_address.as_str())?;

    update_config(
        deps.storage,
        Config {
            admin: msg.admin,
            router_address: msg.router_address,
        },
    )?;

    Ok(Response::new())
}

#[entry_point]
pub fn migrate(deps: DepsMut, _: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let config = get_config(deps.storage)?;

    deps.api.addr_validate(msg.router_address.as_str())?;

    update_config(
        deps.storage,
        Config {
            router_address: msg.router_address,
            ..config
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Swap {
            minimum_receive_amount,
            route,
        } => {
            if route.is_none() {
                return Err(ContractError::Route {});
            }
            swap_handler(deps, env, info, minimum_receive_amount, route.unwrap())
        }
        ExecuteMsg::Receive(receive_msg) => {
            let info = from_cw20_receive_msg(&deps.as_ref(), info, receive_msg.clone())?;
            let msg = from_json(receive_msg.msg)?;

            match msg {
                ExecuteMsg::Receive(_) => {
                    Err(ContractError::Std(cosmwasm_std::StdError::GenericErr {
                        msg: "nested receive not allowed".to_string(),
                    }))
                }
                _ => execute(deps, env, info, msg),
            }
        }
        _ => not_implemented_handle(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTwapToNow {
            swap_denom,
            target_denom,
            period,
            route,
        } => {
            if route.is_none() {
                return Err(ContractError::Route {}.into());
            }
            to_json_binary(&get_twap_to_now_handler(
                deps,
                swap_denom,
                target_denom,
                period,
                &route.unwrap(),
            )?)
        }
        QueryMsg::GetExpectedReceiveAmount {
            swap_amount,
            target_denom,
            route,
        } => {
            if route.is_none() {
                return Err(ContractError::Route {}.into());
            }
            to_json_binary(&get_expected_receive_amount_handler(
                deps,
                swap_amount,
                target_denom,
                &route.unwrap(),
            )?)
        }
        _ => to_json_binary(&not_implemented_query()?),
    }
}

pub const AFTER_SWAP: u64 = 1;
pub const AFTER_SUBMIT_ORDER: u64 = 2;
pub const AFTER_RETRACT_ORDER: u64 = 3;
pub const AFTER_WITHDRAW_ORDER: u64 = 4;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_SWAP => return_swapped_funds(deps.as_ref(), env),
        AFTER_SUBMIT_ORDER => not_implemented_handle(),
        AFTER_RETRACT_ORDER => not_implemented_handle(),
        AFTER_WITHDRAW_ORDER => not_implemented_handle(),
        _ => Err(ContractError::MissingReplyId {}),
    }
}

pub fn not_implemented_query() -> StdResult<()> {
    Err(cosmwasm_std::StdError::GenericErr {
        msg: "not implemented".to_string(),
    })
}

pub fn not_implemented_handle() -> Result<Response, ContractError> {
    Err(ContractError::Std(not_implemented_query().unwrap_err()))
}
