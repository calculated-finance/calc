use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_name: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    AssignFundCore { fund_core_address: Addr },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FundCoreResponse)]
    GetFundCore {},
}

#[cw_serde]
pub struct FundCoreResponse {
    pub address: Addr,
}
