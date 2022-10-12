use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cache {
    pub owner: Addr,
    pub funds: Coin,
    pub validator_address: Addr,
}

pub const CACHE: Item<Cache> = Item::new("cache_v1");
pub const COMPOUNDER_CONTRACT_CODE_ID: Item<u64> = Item::new("compounder_contract_code_id_v1");
pub const COMPOUNDER_CONTRACTS_BY_ADDRESS: Map<Addr, Addr> =
    Map::new("compounder_contracts_by_address_v1");
