use base::pair::Pair;
use cw_storage_plus::Map;
use osmosis_helpers::pool::Pool;

pub const POOLS: Map<u64, Pool> = Map::new("pools_v1");
