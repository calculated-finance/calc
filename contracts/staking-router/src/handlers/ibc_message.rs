use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use cosmos_sdk_proto::ibc::applications::transfer::v1::MsgTransfer;
use cosmos_sdk_proto::traits::Message;
use cosmwasm_std::{Binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response};

use crate::validation_helpers::assert_exactly_one_asset;
use crate::ContractError;

pub fn ibc_transfer(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    port_id: String,
    to_address: String,
    timeout: u64
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;

    let msg = create_transfer_message(env, channel_id, port_id, to_address, info.funds[0].clone(), timeout);
    Ok(Response::new()
        .add_attribute("method", "ibc_transfer")
        .add_message(msg))
}

fn create_transfer_message(
    env: Env,
    channel_id: String,
    port_id: String,
    to_address: String,
    coin: Coin,
    timeout: u64
) -> CosmosMsg {
    let mut buffer = vec![];

    MsgTransfer {
        receiver: to_address,
        sender: env.contract.address.into(),
        source_channel: channel_id,
        source_port: port_id,
        timeout_height: None,
        timeout_timestamp: timeout,
        token: Some(ProtoCoin {
            denom: coin.denom.clone(),
            amount: coin.amount.to_string(),
        }),
    }
    .encode(&mut buffer)
    .unwrap();

    CosmosMsg::Stargate {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: Binary::from(buffer),
    }
}
