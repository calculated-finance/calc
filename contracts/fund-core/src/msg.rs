use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Decimal256};
use std::collections::HashMap;

#[cw_serde]
pub struct InstantiateMsg {
    pub router: Addr,
    pub swapper: Addr,
    pub base_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Rebalance {
        allocations: HashMap<String, Decimal>,
        slippage_tolerance: Option<Decimal256>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
