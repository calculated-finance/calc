use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use osmosis_std::types::osmosis::gamm::v1beta1::{
    QueryPoolResponse, QuerySwapExactAmountInResponse,
};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Swap { pool_id: u64, denom_out: String },
    ProvideLiquidityAndLockTokens { pool_id: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QueryPoolResponse)]
    GetPool { pool_id: u64 },
    #[returns(QuerySwapExactAmountInResponse)]
    GetPrice {
        pool_id: u64,
        denom_in: String,
        amount_in: Uint128,
        denom_out: String,
    },
}
