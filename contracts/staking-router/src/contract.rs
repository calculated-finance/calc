#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::handlers::add_allowed_z_caller::add_allowed_z_caller;
use crate::handlers::get_allowed_z_callers::get_allowed_z_callers;
use crate::handlers::remove_allowed_z_caller::remove_allowed_z_caller;
use crate::handlers::test_ibc::test_ibc;
use crate::handlers::zdelegate::zdelegate;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:staking-router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    CONFIG.remove(deps.storage);

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: msg.admin,
        allowed_z_callers: msg.allowed_z_callers,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: msg.admin,
        allowed_z_callers: msg.allowed_z_callers,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ZDelegate {
            delegator_address,
            validator_address,
            denom,
            amount,
        } => zdelegate(
            deps,
            env,
            info,
            delegator_address,
            validator_address,
            denom,
            amount,
        ),
        ExecuteMsg::AddAllowedZCaller { allowed_z_caller } => {
            add_allowed_z_caller(deps, info, allowed_z_caller)
        }
        ExecuteMsg::RemoveAllowedZCaller { allowed_z_caller } => {
            remove_allowed_z_caller(deps, info, allowed_z_caller)
        },
        ExecuteMsg::TestIBC { value, channel_id } => test_ibc(deps, env, value, channel_id)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::GetAllowedZCallers {} => to_binary(&get_allowed_z_callers(deps)?),
    }
}
