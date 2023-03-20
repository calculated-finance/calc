use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        pool_id: u64
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
