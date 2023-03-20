use cosmwasm_schema::{cw_serde, QueryResponses};
use osmosis_std::types::osmosis::gamm::v1beta1::QueryPoolResponse;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Swap { pool_id: u64, denom_out: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QueryPoolResponse)]
    GetPool { pool_id: u64 },
}
