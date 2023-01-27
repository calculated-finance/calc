use crate::msg::{BowPoolsQueryMsg, BowPoolsResponse};
use cosmwasm_std::{Addr, QuerierWrapper, StdResult};

pub fn query_pool_balances(
    querier: QuerierWrapper,
    pool_address: Addr,
) -> StdResult<BowPoolsResponse> {
    let pool_balances_response: BowPoolsResponse =
        querier.query_wasm_smart(pool_address, &BowPoolsQueryMsg::Pool {})?;

    Ok(pool_balances_response)
}
