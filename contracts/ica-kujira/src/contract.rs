use base::ibc::msg::CalcIBC;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, IbcMsg, to_binary, IbcTimeout, SubMsg, CosmosMsg, Reply};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ica-kujira";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: InstantiateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::Test { channel_id, value } => handle_test(_env, channel_id, value)
    }
}

pub fn handle_test(env: Env, channel_id: String, value: String) -> Result<Response, ContractError> {

    let packet = CalcIBC::Test { value };

    let ibc_packet = IbcMsg::SendPacket { channel_id, data: to_binary(&packet)?, timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300)) };

    let msg = SubMsg::reply_always(CosmosMsg::Ibc(ibc_packet), 1);

    Ok(
        Response::new()
        .add_attribute("method", "handle_test")
        .add_submessage(msg)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        1 => {
            Ok(
                Response::new()
                .add_attribute("reply", "yoyoy")
            )
        },
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
