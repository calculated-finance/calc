use super::{callback::Callback, pair::Pair};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Swap {
    pub path: Vec<Pair>,
    pub callback: Callback,
}
