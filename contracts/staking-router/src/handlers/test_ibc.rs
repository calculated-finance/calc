use base::ibc::msg::CalcIBC;
use cosmwasm_std::{DepsMut, Response, IbcMsg, to_binary, Env};

use crate::ContractError;

pub fn test_ibc(_deps: DepsMut, env: Env, value: String, channel_id: String) -> Result<Response, ContractError> {

    let data = CalcIBC::Test { value };

    let msg = IbcMsg::SendPacket { channel_id, data: to_binary(&data)?, timeout: env.block.time.plus_seconds(120).into() };

    Ok(
        Response::new()
        .add_attribute("method", "test_ibc")
        .add_message(msg)
    )
}