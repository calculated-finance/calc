use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use serde::{Deserialize, Serialize};

#[cw_serde]
pub enum BowPoolsQueryMsg {
    Config {},
    Pool {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BowPoolsResponse {
    pub balances: [Uint128; 2],
}

#[cw_serde]
pub enum BowPoolsExecuteMsg {
    Deposit {},
    Withdraw {},
}

#[cw_serde]
pub enum BowStakingExecuteMsg {
    Stake {},
    Withdraw { amount: Coin },
}
