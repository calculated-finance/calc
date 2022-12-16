#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::handlers::add_allowed_z_caller::add_allowed_z_caller;
use crate::handlers::after_ibc_transfer::after_ibc_transfer;
use crate::handlers::get_allowed_z_callers::get_allowed_z_callers;
use crate::handlers::ibc_delegate::ibc_delegate;
use crate::handlers::remove_allowed_z_caller::remove_allowed_z_caller;
use crate::handlers::z_delegate::z_delegate;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:staking-router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const AFTER_IBC_TRANSFER_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
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
        admin: deps.api.addr_validate(&msg.admin.to_string())?,
        allowed_z_callers: msg
            .allowed_z_callers
            .iter()
            .map(|caller_address| {
                deps.api.addr_validate(&caller_address.to_string()).expect(
                    &format!("a valid address for allowed z caller {:?}", caller_address)
                        .to_string(),
                )
            })
            .collect(),
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
        ExecuteMsg::IbcDelegate {
            transfer_channel_id,
            packet_channel_id,
            delegator_address,
            validator_address,
        } => ibc_delegate(env, info, transfer_channel_id, packet_channel_id, delegator_address, validator_address),
        ExecuteMsg::ZDelegate {
            delegator_address,
            validator_address,
            denom,
            amount,
        } => z_delegate(
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
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_IBC_TRANSFER_REPLY_ID => after_ibc_transfer(deps, env, reply),
        id => Err(ContractError::CustomError {
            val: format!("unknown reply id: {}", id),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::GetAllowedZCallers {} => to_binary(&get_allowed_z_callers(deps)?),
    }
}
