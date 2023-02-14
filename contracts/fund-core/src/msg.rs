use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub router: Addr,
    pub swapper: Addr,
    pub base_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Migrate { new_fund_address: Addr },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
