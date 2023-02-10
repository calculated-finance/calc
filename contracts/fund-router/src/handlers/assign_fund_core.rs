use base::ContractError;
use cosmwasm_std::{Addr, DepsMut, Response};

use crate::state::fund_cores::save_fund_core;

pub fn assign_fund_core(deps: DepsMut, fund_core_address: Addr) -> Result<Response, ContractError> {
    deps.api.addr_validate(fund_core_address.as_str())?;

    save_fund_core(deps, fund_core_address.clone())?;
    Ok(Response::new()
        .add_attribute("method", "assign_fund_core")
        .add_attribute("fund_core_address", fund_core_address.to_string()))
}
