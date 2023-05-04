use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct VaultCache {
    pub vault_id: Uint128,
}

pub const VAULT_CACHE: Item<VaultCache> = Item::new("vault_cache_v8");

#[cw_serde]
pub struct SwapCache {
    pub swap_denom_balance: Coin,
    pub receive_denom_balance: Coin,
}

pub const SWAP_CACHE: Item<SwapCache> = Item::new("swap_cache_v8");
