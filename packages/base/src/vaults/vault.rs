use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};

#[cw_serde]
pub enum PositionType {
    Enter,
    Exit,
}

#[cw_serde]
#[derive(Copy)]
pub enum VaultStatus {
    Scheduled = 0,
    Active = 1,
    Inactive = 2,
    Cancelled = 3,
}

#[cw_serde]
pub enum PostExecutionAction {
    Send,
    ZDelegate,
}

#[cw_serde]
pub struct Destination {
    pub address: Addr,
    pub allocation: Decimal,
    pub action: PostExecutionAction,
}
