use std::collections::VecDeque;

use cosmwasm_std::{Addr, Deps, DepsMut, StdResult};
use cw_storage_plus::Item;

pub const FUND_CORES: Item<VecDeque<Addr>> = Item::new("fund_cores_v1");

pub fn initialise_fund_cores(deps: DepsMut) -> StdResult<()> {
    let fund_cores: VecDeque<Addr> = VecDeque::new();
    FUND_CORES.save(deps.storage, &fund_cores)?;
    Ok(())
}

pub fn get_current_fund_core(deps: Deps) -> StdResult<Addr> {
    let fund_cores = FUND_CORES.load(deps.storage)?;
    let current_fund_core = fund_cores.front().expect("fund cores should not be empty");
    Ok(current_fund_core.clone())
}

pub fn save_fund_core(deps: DepsMut, fund_core_address: Addr) -> StdResult<Addr> {
    let mut fund_cores = FUND_CORES.load(deps.storage)?;
    fund_cores.push_front(fund_core_address.clone());
    FUND_CORES.save(deps.storage, &fund_cores)?;
    Ok(fund_core_address)
}
