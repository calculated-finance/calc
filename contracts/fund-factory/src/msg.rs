use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub router_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: Option<Addr>,
        router_code_id: Option<u64>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
