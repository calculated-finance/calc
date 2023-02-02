use super::state::fetch_and_increment_counter;
use crate::types::swap::{Swap, SwapBuilder};
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Item, Map};

const SWAP_COUNTER: Item<u64> = Item::new("swap_counter_v1");

const SWAPS: Map<u64, Swap> = Map::new("swaps_v1");

pub fn save_swap(store: &mut dyn Storage, swap_builder: SwapBuilder) -> StdResult<u64> {
    let swap_id = fetch_and_increment_counter(store, SWAP_COUNTER)?;
    SWAPS.save(store, swap_id, &swap_builder.build(swap_id))?;
    Ok(swap_id)
}

pub fn update_swap(store: &mut dyn Storage, swap: Swap) -> StdResult<()> {
    SWAPS.save(store, swap.id, &swap)?;
    Ok(())
}

pub fn delete_swap(store: &mut dyn Storage, swap_id: u64) {
    SWAPS.remove(store, swap_id)
}

pub fn get_swap(store: &dyn Storage, swap_id: u64) -> StdResult<Swap> {
    SWAPS.load(store, swap_id)
}