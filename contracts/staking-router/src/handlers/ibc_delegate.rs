use cosmwasm_std::{Addr, DepsMut, Env, IbcMsg, MessageInfo, Response};

use crate::{validation_helpers::assert_exactly_one_asset, ContractError};

pub fn ibc_delegate(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    staking_contract_address: String,
    _delegator_address: Addr,
    _validator_address: Addr,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;

    let ibc_transfer_msg = IbcMsg::Transfer {
        channel_id,
        to_address: staking_contract_address,
        amount: info.funds.clone().pop().unwrap(),
        timeout: env.block.time.plus_seconds(300).into(),
    };

    Ok(Response::new()
        .add_attribute("method", "ibc_delegate")
        .add_message(ibc_transfer_msg))
}
