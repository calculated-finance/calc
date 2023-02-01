use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal256};
use petgraph::prelude::EdgeIndex;

#[cw_serde]
pub enum UnweightedExchange {
    Fin {
        address: Addr,
        quote_denom: String,
        base_denom: String,
    },
}

impl From<EdgeIndex<UnweightedExchange>> for UnweightedExchange {
    fn from(edge: EdgeIndex<UnweightedExchange>) -> Self {
        edge.into()
    }
}

impl From<EdgeIndex> for UnweightedExchange {
    fn from(edge: EdgeIndex) -> Self {
        edge.into()
    }
}

#[cw_serde]
pub struct WeightedExchange {
    pub exchange: UnweightedExchange,
    pub price: Decimal256,
}

impl From<EdgeIndex<WeightedExchange>> for WeightedExchange {
    fn from(edge: EdgeIndex<WeightedExchange>) -> Self {
        edge.into()
    }
}

impl From<EdgeIndex> for WeightedExchange {
    fn from(edge: EdgeIndex) -> Self {
        edge.into()
    }
}
