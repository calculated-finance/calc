use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};

#[cw_serde]
pub enum PositionType {
    Enter,
    Exit,
}

#[cw_serde]
pub enum VaultStatus {
    Scheduled,
    Active,
    Inactive,
    Cancelled,
}

#[cw_serde]
pub enum PostExecutionActionDeprecated {
    Send,
    ZDelegate,
}

#[cw_serde]
pub enum PostExecutionAction {
    Send,
    ZDelegate,
    IbcDelegate {
        transfer_channel_id: String,
        packet_channel_id: String,
        validator_address: String
    }
}

#[cw_serde]
pub struct DestinationDeprecated {
    pub address: Addr,
    pub allocation: Decimal,
    pub action: PostExecutionActionDeprecated,
}

#[cw_serde]
pub struct Destination {
    pub address: Addr, // for ibc staking this will be the users wallet on the destination chain
    pub allocation: Decimal,
    pub action: PostExecutionAction,
}
