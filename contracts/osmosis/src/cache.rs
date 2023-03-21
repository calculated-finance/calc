use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Cache {
    pub pool_id: u64,
}

impl From<u64> for Cache {
    fn from(pool_id: u64) -> Self {
        Self { pool_id }
    }
}

pub const CACHE: Item<Cache> = Item::new("cache");
