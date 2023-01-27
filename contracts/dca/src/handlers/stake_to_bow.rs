use crate::{
    error::ContractError,
    state::{
        cache::{BOW_CACHE, CACHE},
        config::get_config,
        vaults::get_vault,
    },
};
use bow_helpers::msg::BowStakingExecuteMsg;
use cosmos_sdk_proto::{
    cosmos::{authz::v1beta1::MsgExec, base::v1beta1::Coin},
    cosmwasm::wasm::v1::MsgExecuteContract,
    traits::Message,
    Any,
};
use cosmwasm_std::{to_binary, Binary, CosmosMsg, DepsMut, Env, Response};

pub fn stake_to_bow(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let cache = BOW_CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, CACHE.load(deps.storage)?.vault_id.into())?;

    let lp_token_balance = cache.lp_token_balance.expect("LP token balance");

    let amount_to_stake = Coin {
        denom: lp_token_balance.denom,
        amount: lp_token_balance.amount.to_string(),
    };

    let config = get_config(deps.storage)?;

    let mut buffer = vec![];

    MsgExecuteContract {
        contract: config.bow_staking_address.to_string(),
        sender: vault.owner.to_string(),
        msg: to_binary(&BowStakingExecuteMsg::Stake {}).unwrap().to_vec(),
        funds: vec![amount_to_stake.clone()],
    }
    .encode(&mut buffer)
    .unwrap();

    Ok(Response::new()
        .add_attribute("lp_tokens_staked", format!("{:?}", amount_to_stake))
        .add_message(CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
            value: Binary(
                MsgExec {
                    grantee: env.contract.address.to_string(),
                    msgs: vec![Any {
                        type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".to_string(),
                        value: buffer,
                    }],
                }
                .encode_to_vec(),
            ),
        }))
}
