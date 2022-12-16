use crate::{contract::AFTER_IBC_TRANSFER_REPLY_ID, ContractError};
use base::ibc::delegation::DelegationPacket;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, Env, IbcTimeout, MessageInfo, Response, SubMsg};

pub fn ibc_delegate(
    env: Env,
    info: MessageInfo,
    transfer_channel_id: String,
    packet_channel_id: String,
    delegator_address: String,
    validator_address: String,
) -> Result<Response, ContractError> {
    let ibc_transfer_msg = create_transfer_msg(
        env.clone(),
        transfer_channel_id.clone(),
        delegator_address,
        info.funds[0].clone(),
    );
    let ibc_delegation_msg = create_delegation_msg(env, packet_channel_id.clone(), validator_address);
    Ok(Response::new()
        .add_attribute("method", "ibc_delegate")
        .add_submessage(ibc_transfer_msg)
        .add_message(ibc_delegation_msg))
}

fn create_transfer_msg(env: Env, channel_id: String, to_address: String, amount: Coin) -> SubMsg {
    SubMsg::reply_always(
        CosmosMsg::Ibc(cosmwasm_std::IbcMsg::Transfer {
            channel_id,
            to_address,
            amount: amount.clone(),
            timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300)),
        }),
        AFTER_IBC_TRANSFER_REPLY_ID,
    )
}

fn create_delegation_msg(env: Env, channel_id: String, validator_address: String) -> CosmosMsg {
    let ibc_delegation_packet = DelegationPacket { validator_address };

    CosmosMsg::Ibc(cosmwasm_std::IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&ibc_delegation_packet).unwrap(),
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(300)),
    })
}
