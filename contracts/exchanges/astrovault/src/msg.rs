use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};
use exchange::msg::Pair;
use exchange_macros::{exchange_execute, exchange_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub router_address: Addr,
}

#[cw_serde]
pub struct MigrateMsg {
    pub router_address: Addr,
}

#[exchange_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[exchange_execute]
#[cw_serde]
pub enum ExecuteMsg {}
