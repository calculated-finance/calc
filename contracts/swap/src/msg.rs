use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal256, Uint128};

use crate::types::callback::Callback;

#[cw_serde]
pub struct MigrateMsg {
    pub admin: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        belief_price: Option<Decimal256>,
        max_spread: Option<Decimal256>,
        callback: Callback,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // #[returns(Vec<Addr>)]
    // GetAllowedZCallers {},
}
