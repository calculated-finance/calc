use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Decimal256};

#[cw_serde]
pub struct InstantiateMsg {}

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
