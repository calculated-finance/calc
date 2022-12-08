use cosmos_sdk_proto::{traits::Message};
use cosmwasm_std::{Response, CosmosMsg, Binary};

use crate::ContractError;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterAccount {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub connection_id: ::prost::alloc::string::String,
}

pub fn register_ica(
    owner: String,
    connection_id: String,
    version: String,
    type_url: String
) -> Result<Response, ContractError> {

    

    let mut buffer = vec![];
    MsgRegisterAccount {
        owner,
        connection_id,
    }
    .encode(&mut buffer)
    .unwrap();

    let msg = CosmosMsg::Stargate {
        type_url,
        value: Binary::from(buffer),
    };

    ///ibc.applications.interchain_accounts.controller.v1.MsgRegisterInterchainAccount".to_string(),
    /// 
    Ok(
        Response::new()
        .add_message(msg)
    )
}

