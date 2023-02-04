use crate::types::callback::Callback;
use crate::{state::config::Config, types::pair::Pair};
use base::pair::Pair as FinPair;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Decimal256};

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
    AddPair {
        pair: Pair,
    },
    CreateSwap {
        target_denom: String,
        slippage_tolerance: Option<Decimal256>,
        callback: Callback,
    },
    ContinueSwap {
        swap_id: u64,
    },
    SwapOnFin {
        pair: FinPair,
        slippage_tolerance: Option<Decimal256>,
        callback: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},
    #[returns(Vec<Pair>)]
    GetPath { denoms: [String; 2] },
}
