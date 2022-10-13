use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cache {
    pub funds: Coin,
    pub validator_address: Addr,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub admin: Addr,
}

#[derive(Serialize, Deserialize)]
pub struct CompoundingDelegation {
    pub validator_address: Addr,
    pub balance: Coin,
}

pub const CACHE: Item<Cache> = Item::new("cache_v1");
pub const DELEGATIONS: Map<Addr, CompoundingDelegation> = Map::new("compounding_delegations_v1");
pub const CONFIG: Item<Config> = Item::new("config_v1");
