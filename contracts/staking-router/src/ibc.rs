use cosmwasm_std::{
    entry_point, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, StdResult,
};

use crate::{
    error::ContractError,
    state::{Config, CONFIG},
};

#[entry_point]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    Ok(None)
}

#[entry_point]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_channel_connect"))
}

#[entry_point]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_channel_close"))
}

#[entry_point]
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    Ok(IbcReceiveResponse::new().add_attribute("method", "ibc_packet_receive"))
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    CONFIG.update(deps.storage, |mut existing_config| -> StdResult<Config> {
        existing_config.counter = existing_config.counter + 1;
        Ok(existing_config)
    })?;

    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_ack"))
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_packet_timeout"))
}
