use super::exchange::UnweightedExchange;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal256;
use std::collections::VecDeque;

#[cw_serde]
pub struct Path {
    pub cost: Decimal256,
    pub exchanges: VecDeque<UnweightedExchange>,
}
