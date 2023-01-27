use crate::{
    contract::AFTER_SENDING_LP_TOKENS_TO_CONTRACT,
    error::ContractError,
    state::{cache::CACHE, vaults::get_vault},
};
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use cosmos_sdk_proto::{
    cosmos::authz::v1beta1::MsgExec, cosmos::bank::v1beta1::MsgSend, traits::Message, Any,
};
use cosmwasm_std::{Binary, CosmosMsg, DepsMut, Env, Response, SubMsg};

pub fn send_lp_tokens_to_contract(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let vault = get_vault(deps.storage, CACHE.load(deps.storage)?.vault_id.into())?;
    let mut buffer = vec![];

    MsgSend {
        from_address: vault.owner.to_string(),
        to_address: env.contract.address.to_string(),
        amount: vec![ProtoCoin {
            amount: vault.balance.amount.to_string(),
            denom: vault.balance.denom.clone(),
        }],
    }
    .encode(&mut buffer)
    .unwrap();

    let send_lp_tokens_to_contract_message = SubMsg::reply_on_success(
        CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
            value: Binary(
                MsgExec {
                    grantee: env.contract.address.to_string(),
                    msgs: vec![Any {
                        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                        value: buffer,
                    }],
                }
                .encode_to_vec(),
            ),
        },
        AFTER_SENDING_LP_TOKENS_TO_CONTRACT,
    );

    Ok(Response::new()
        .add_attribute("lp_tokens_undelegated", format!("{:?}", vault.balance))
        .add_submessage(send_lp_tokens_to_contract_message))
}
