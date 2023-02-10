use cosmwasm_std::{Deps, StdResult};

use crate::{msg::FundCoreResponse, state::fund_cores::get_current_fund_core};

pub fn get_fund_core(deps: Deps) -> StdResult<FundCoreResponse> {
    get_current_fund_core(deps).map(|fund_core_address| FundCoreResponse {
        address: fund_core_address,
    })
}
