use crate::state::State;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(State)]
    GetState {},
}

#[cw_serde]
pub enum CalcIBC {
    Test { value: String },
}

#[cw_serde]
pub struct TestResponse {
    pub value: String,
}
