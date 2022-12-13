use cosmwasm_std::{DepsMut, Env, Reply, Response, SubMsgResult};

use crate::ContractError;

pub fn after_ibc_transfer(_deps: DepsMut, _env: Env, _reply: Reply) -> Result<Response, ContractError> {
    match _reply.result {
        SubMsgResult::Ok(_) => {
        },
        SubMsgResult::Err(_) => {

        }
    }
    Ok(
        Response::new()
    )
}