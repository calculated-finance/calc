use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::validation_helpers::assert_sender_is_allowed_z_caller;
use crate::ContractError;
use base::authz::helpers::{create_exec_message, create_protobuf_delegate_msg};

pub fn z_delegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    delegator_address: Addr,
    validator_address: Addr,
    denom: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    assert_sender_is_allowed_z_caller(deps.storage, info.sender)?;

    let protobuf_delegate_msg =
        create_protobuf_delegate_msg(delegator_address, validator_address, denom, amount);
    let msg = create_exec_message(env.contract.address, protobuf_delegate_msg);
    Ok(Response::new()
        .add_attribute("method", "zdelegate")
        .add_message(msg))
}
