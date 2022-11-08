use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Cache {
    pub vault_id: Uint128,
    pub owner: Addr,
    pub total_automation_fees: Option<Coin>,
    pub refunded: Option<bool>,
}

#[cw_serde]
pub struct LimitOrderCache {
    pub order_idx: Uint128,
    pub offer_amount: Uint128,
    pub original_offer_amount: Uint128,
    pub filled: Uint128,
}

pub const CACHE: Item<Cache> = Item::new("cache_v4");

pub const LIMIT_ORDER_CACHE: Item<LimitOrderCache> = Item::new("limit_order_cache_v4");
