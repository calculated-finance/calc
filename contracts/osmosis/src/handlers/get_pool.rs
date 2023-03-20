use cosmwasm_std::{Deps, StdResult};
use osmosis_std::types::osmosis::gamm::v1beta1::{QueryPoolRequest, QueryPoolResponse};

pub fn get_pool(deps: Deps, pool_id: u64) -> StdResult<QueryPoolResponse> {
    QueryPoolRequest { pool_id }.query(&deps.querier)
}
