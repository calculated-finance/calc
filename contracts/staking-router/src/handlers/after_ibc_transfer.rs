use cosmwasm_std::{DepsMut, Env, Reply, Response, SubMsgResult};

use crate::ContractError;

pub fn after_ibc_transfer(
    _deps: DepsMut,
    _env: Env,
    _reply: Reply,
) -> Result<Response, ContractError> {
    match _reply.result {
        SubMsgResult::Ok(_) => Ok(Response::new()),
        SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!("failed to submit ibc transfer: {}", e),
        }),
    }
}
