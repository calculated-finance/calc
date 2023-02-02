use std::collections::VecDeque;

use super::{callback::Callback, exchange::Exchange};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct Swap {
    pub id: u64,
    pub path: VecDeque<Exchange>,
    pub callback: Callback,
    pub balance: Coin,
}

pub struct SwapBuilder {
    pub path: VecDeque<Exchange>,
    pub callback: Callback,
    pub balance: Coin,
}

impl SwapBuilder {
    pub fn new(path: VecDeque<Exchange>, callback: Callback, balance: Coin) -> Self {
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
