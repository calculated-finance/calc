use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cache {
    pub owner: Addr,
    pub funds: Coin,
    pub validator_address: Addr,
}

pub const CACHE: Item<Cache> = Item::new("cache_v1");
pub const CONTRACT_CODE_ID: Item<u64> = Item::new("contract_code_id_v1");
pub const CONTRACTS_BY_ADDRESS: Map<Addr, Addr> = Map::new("contracts_by_address_v1");
