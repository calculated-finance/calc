use std::collections::VecDeque;

use super::{callback::Callback, exchange::UnweightedExchange};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct Swap {
    pub id: u64,
    pub path: VecDeque<UnweightedExchange>,
    pub callback: Callback,
    pub balance: Coin,
}

pub struct SwapBuilder {
    pub path: VecDeque<UnweightedExchange>,
    pub callback: Callback,
    pub balance: Coin,
}

impl SwapBuilder {
    pub fn new(path: VecDeque<UnweightedExchange>, callback: Callback, balance: Coin) -> Self {
        Self {
            path,
            callback,
            balance,
        }
    }

    pub fn build(self, id: u64) -> Swap {
        Swap {
            id,
            path: self.path,
            callback: self.callback,
            balance: self.balance,
        }
    }
}
