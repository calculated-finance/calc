use base::pool::Pool;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Map;

const POOLS: Map<Addr, Pool> = Map::new("pools_v1");

pub fn save_pool(storage: &mut dyn Storage, pool: &Pool) -> StdResult<()> {
    POOLS.save(storage, pool.address.clone(), pool)
}

pub fn get_pool(storage: &dyn Storage, address: &Addr) -> StdResult<Option<Pool>> {
    Ok(POOLS.may_load(storage, address.clone())?)
}
