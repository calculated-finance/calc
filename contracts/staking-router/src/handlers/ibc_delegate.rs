use cosmos_sdk_proto::{ibc::applications::transfer::v2::FungibleTokenPacketData, traits::Message};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, Binary, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Response, Uint128, IbcQuery,
};

use crate::{validation_helpers::assert_exactly_one_asset, ContractError};

#[derive(Default)]
#[cw_serde]
pub struct Ics20Packet {
    /// amount of tokens to transfer is encoded as a string, but limited to u64 max
    pub amount: Uint128,
    /// the token denomination to be transferred
    pub denom: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// the sender address
    pub sender: String,
}

pub fn ibc_send_packet(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    staking_contract_address: String,
    _delegator_address: Addr,
    _validator_address: Addr,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;

    let coin_to_send = info.funds[0].clone();

    let ics20 = Ics20Packet {
        amount: coin_to_send.amount.clone(),
        denom: coin_to_send.denom.clone(),
        sender: env.contract.address.to_string(),
        receiver: staking_contract_address,
    };

    let ibc_packet = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&ics20)?,
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300000)),
    };

    Ok(Response::new()
        .add_attribute("method", "ibc_delegate")
        .add_message(ibc_packet))
}


pub fn ibc_send_fungible(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    staking_contract_address: String,
    _delegator_address: Addr,
    _validator_address: Addr,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;

    let coin_to_send = info.funds[0].clone();
    let mut buffer = vec![];

    FungibleTokenPacketData {
        amount: coin_to_send.amount.to_string(),
        denom: coin_to_send.denom.clone(),
        sender: env.contract.address.to_string(),
        receiver: staking_contract_address,
    }
    .encode(&mut buffer)
    .unwrap();

    let ibc_packet = IbcMsg::SendPacket {
        channel_id,
        data: Binary::from(buffer),
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300000)),
    };

    Ok(Response::new()
        .add_attribute("method", "ibc_delegate")
        .add_message(ibc_packet))
}

pub fn ibc_send_tokens(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    to_address: String,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;

    let coin_to_send = info.funds[0].clone();

    let ibc_send_msg = IbcMsg::Transfer {
        channel_id,
        to_address,
        amount: coin_to_send,
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300)),
    };

    Ok(Response::new()
        .add_attribute("method", "ibc_delegate")
        .add_message(ibc_send_msg))
}
