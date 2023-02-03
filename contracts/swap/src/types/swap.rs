use std::collections::VecDeque;

use super::{callback::Callback, exchange::Pair};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct Swap {
    pub id: u64,
    pub path: VecDeque<Pair>,
    pub callback: Callback,
    pub balance: Coin,
}
