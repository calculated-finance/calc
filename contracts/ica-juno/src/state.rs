use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    pub value: String,
    pub caller: String,
}

pub const STATE: Item<State> = Item::new("state");
