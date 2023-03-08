use crate::{
    msg::{ExecuteMsg, InternalExecuteMsg},
    types::vault::Vault,
};
use cosmwasm_std::{to_binary, CosmosMsg, Empty, Env, SubMsg, WasmMsg};

pub fn create_claim_escrowed_funds_message(vault: &Vault, env: Env) -> Option<SubMsg> {
    vault.dca_plus_config.clone().map_or(None, |_| {
        Some(SubMsg::new(CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::InternalExecuteMsg {
                msg: to_binary(&InternalExecuteMsg::ClaimEscrowedFunds { vault_id: vault.id })
                    .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        })))
    })
}
