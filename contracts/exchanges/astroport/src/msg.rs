use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};
use exchange::msg::Pair;
use exchange_macros::{exchange_execute, exchange_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub router_address: Addr,
}

pub type MigrateMsg = InstantiateMsg;

#[exchange_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[exchange_execute]
#[cw_serde]
pub enum ExecuteMsg {}
