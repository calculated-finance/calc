use cosmwasm_std::{
    entry_point, DepsMut, Env, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcReceiveResponse, StdResult,
};

use crate::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelOpenMsg,
) -> Result<(), ContractError> {
    //let channel = msg.channel();
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// once it's established, we send a WhoAmI message
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    // let channel = msg.channel();
    // let channel_id = &channel.endpoint.channel_id;
    Ok(IbcBasicResponse::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// On closed channel, simply delete the account from our local store
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    //let channel = msg.channel();
    Ok(IbcBasicResponse::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// never should be called as the other side never sends packets
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    Ok(IbcReceiveResponse::new().set_ack(b"{}"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // which local channel was this packet send from
    //let caller = msg.original_packet.src.channel_id.clone();
    // we need to parse the ack based on our request

    Ok(IbcBasicResponse::new())
}
