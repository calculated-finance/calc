use base::ContractError;
use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response};

use crate::{state::funds::save_fund, validation_helpers::assert_sender_is_factory};

pub fn assign_fund(
    deps: DepsMut,
    info: MessageInfo,
    fund_address: Addr,
) -> Result<Response, ContractError> {
    assert_sender_is_factory(deps.storage, info.sender.clone())?;
    deps.api.addr_validate(fund_address.as_str())?;

    save_fund(deps.storage, fund_address.clone())?;
    Ok(Response::new()
        .add_attribute("method", "assign_fund")
        .add_attribute("fund_address", fund_address.to_string()))
}
