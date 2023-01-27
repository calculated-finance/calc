use base::pair::Pair;
use cosmwasm_std::{Addr, Order, StdError, StdResult, Storage};
use cw_storage_plus::Map;

pub const PAIRS: Map<Addr, Pair> = Map::new("pairs_v1");

pub fn find_pair(store: &dyn Storage, denoms: [String; 2]) -> StdResult<Pair> {
    let pair = PAIRS
        .range(store, None, None, Order::Ascending)
        .map(|result| result.map(|(_address, pair)| pair).expect("stored pair"))
        .find(|pair| denoms.contains(&pair.base_denom) && denoms.contains(&pair.quote_denom));

    pair.ok_or(StdError::generic_err(format!(
        "Pair for denoms {:?} not supported",
        denoms
    )))
}
