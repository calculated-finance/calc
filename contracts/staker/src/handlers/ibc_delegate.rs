use crate::{state::CONFIG, ContractError};
use cosmwasm_std::{to_binary, DepsMut, Env, IbcReceiveResponse, Uint128, Addr};

use base::authz::helpers::{create_exec_message, create_protobuf_delegate_msg};
use base::ibc::msg::IbcAcks;

pub fn ibc_delegate(
    deps: DepsMut,
    env: Env,
    delegator_address: String,
    validator_address: String,
    amount: Uint128,
) -> Result<IbcReceiveResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let protobuf_delegate_msg = create_protobuf_delegate_msg(
        deps.api.addr_validate(&delegator_address)?,
        Addr::unchecked(&validator_address),
        config.native_denom,
        amount,
    );

    let msg = create_exec_message(env.contract.address, protobuf_delegate_msg);

    let ack = IbcAcks::Delegate {};

    Ok(IbcReceiveResponse::new()
        .set_ack(to_binary(&ack)?)
        .add_attribute("method", "zdelegate")
        .add_message(msg))
}
