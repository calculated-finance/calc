#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::handlers::add_allowed_z_caller::add_allowed_z_caller;
use crate::handlers::get_allowed_z_callers::get_allowed_z_callers;
use crate::handlers::get_config::get_config;
use crate::handlers::ibc_delegate::{ibc_send_packet, ibc_send_tokens, ibc_send_fungible};
use crate::handlers::ibc_message::ibc_transfer;
use crate::handlers::remove_allowed_z_caller::remove_allowed_z_caller;
use crate::handlers::zdelegate::zdelegate;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:staking-router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
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
        counter: 0,
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
        ExecuteMsg::IbcTransfer { channel_id, port_id, to_address, timeout } => ibc_transfer(deps, env, info, channel_id, port_id, to_address, timeout),
        ExecuteMsg::IbcSendTokens {
            channel_id,
            to_address,
        } => ibc_send_tokens(deps, env, info, channel_id, to_address),
        ExecuteMsg::IbcDelegate {
            channel_id,
            staking_contract_address,
            delegator_address,
            validator_address,
        } => ibc_send_packet(
            deps,
            env,
            info,
            channel_id,
            staking_contract_address,
            delegator_address,
            validator_address,
        ),
        ExecuteMsg::IbcSendFungible {
            channel_id,
            staking_contract_address,
            delegator_address,
            validator_address,
        } => ibc_send_fungible(
            deps,
            env,
            info,
            channel_id,
            staking_contract_address,
            delegator_address,
            validator_address,
        ),
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
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::GetAllowedZCallers {} => to_binary(&get_allowed_z_callers(deps)?),
        QueryMsg::GetConfig {} => to_binary(&get_config(deps)?),
    }
}
