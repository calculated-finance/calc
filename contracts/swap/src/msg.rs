use crate::types::callback::Callback;
use crate::types::path::Path;
use crate::{state::config::Config, types::exchange::UnweightedExchange};
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
        exchange: UnweightedExchange,
    },
    Swap {
        target_denom: String,
        slippage_tolerance: Option<Decimal256>,
        callback: Callback,
        path: Option<Path>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},
    #[returns(Vec<UnweightedExchange>)]
    GetPath { denoms: [String; 2] },
}
