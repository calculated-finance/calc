use super::claim_escrowed_funds::claim_escrowed_funds_handler;
use crate::{
    error::ContractError, helpers::validation_helpers::assert_sender_is_contract,
    msg::InternalExecuteMsg,
};
use cosmwasm_std::{from_binary, Binary, DepsMut, Env, MessageInfo, Response};

pub fn internal_execute_message_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    message: Binary,
) -> Result<Response, ContractError> {
    assert_sender_is_contract(&info.sender, &env)?;

    from_binary(&message).map_or(
        Err(ContractError::CustomError {
            val: "Message binary not valid".to_string(),
        }),
        |message| match message {
            InternalExecuteMsg::ClaimEscrowedFunds { vault_id } => {
                claim_escrowed_funds_handler(deps, env, vault_id)
            }
        },
    )
}
