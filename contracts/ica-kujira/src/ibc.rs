use base::ibc::msg::{CalcIBC, KCalc};
use cosmwasm_std::{
    entry_point, from_binary, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcReceiveResponse, StdResult, IbcPacketTimeoutMsg,
};

use crate::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    //let channel = msg.channel();
    Ok(None)
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
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_channel_connect"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// On closed channel, simply delete the account from our local store
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    //let channel = msg.channel();
    Ok(IbcBasicResponse::new().add_attribute("method", "ibc_channel_close"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// never should be called as the other side never sends packets
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let packet: KCalc = from_binary(&msg.packet.data)?;
    match packet {
        KCalc::TestResponse { value } => handle_kcalc(value)
    }
}

pub fn handle_kcalc(value: String) -> StdResult<IbcReceiveResponse> {
    Ok(
        IbcReceiveResponse::new()
        .add_attribute("method", "handle_kcalc")
        .add_attribute("value", value.to_string())
    )
}

// this wont be called because we wont send any packets from the juno side for poc
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

#[cfg_attr(not(feature = "library"), entry_point)]
/// we just ignore these now. shall we store some info?
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}