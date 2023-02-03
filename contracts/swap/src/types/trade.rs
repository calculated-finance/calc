use std::collections::VecDeque;

use super::{callback::Callback, swap::Pair};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct Trade {
    pub id: u64,
    pub path: VecDeque<Pair>,
    pub callback: Callback,
    pub balance: Coin,
}

pub struct TradeBuilder {
    pub path: VecDeque<Pair>,
    pub callback: Callback,
    pub balance: Coin,
}

impl TradeBuilder {
    pub fn new(path: VecDeque<Pair>, callback: Callback, balance: Coin) -> Self {
        Self {
            path,
            callback,
            balance,
        }
    }

    pub fn build(self, id: u64) -> Trade {
        Trade {
            id,
            path: self.path,
            callback: self.callback,
            balance: self.balance,
        }
    }
}
