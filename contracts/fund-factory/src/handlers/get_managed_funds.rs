use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{msg::ManagedFundsResponse, state::fund_routers::get_fund_routers_by_address};

pub fn get_managed_funds(deps: Deps, address: Addr) -> StdResult<ManagedFundsResponse> {
    get_fund_routers_by_address(deps.storage, address)
}
