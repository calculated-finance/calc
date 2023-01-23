use crate::types::callback::Callback;
use crate::{state::config::Config, types::exchange::Exchange};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal256};

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
    UpdateConfig {
        admin: Addr,
        paused: bool,
    },
    AddPath {
        denoms: [String; 2],
        exchange: Exchange,
    },
    Swap {
        target_denom: String,
        slippage_tolerance: Option<Decimal256>,
        callback: Callback,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},
    #[returns(Vec<Exchange>)]
    GetPath { denoms: [String; 2] },
}
