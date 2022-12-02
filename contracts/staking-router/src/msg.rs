use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
#[cw_serde]
pub struct MigrateMsg {
    pub admin: Addr,
    pub allowed_z_callers: Vec<Addr>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub allowed_z_callers: Vec<Addr>,
}

#[cw_serde]
pub enum ExecuteMsg {
    IbcTransfer {
        channel_id: String,
        port_id: String,
        to_address: String,
        timeout: u64
    },
    IbcSendTokens {
        channel_id: String,
        to_address: String,
    },
    IbcDelegate {
        channel_id: String,
        staking_contract_address: String,
        delegator_address: Addr,
        validator_address: Addr,
    },
    IbcSendFungible {
        channel_id: String,
        staking_contract_address: String,
        delegator_address: Addr,
        validator_address: Addr,
    },
    ZDelegate {
        delegator_address: Addr,
        validator_address: Addr,
        denom: String,
        amount: Uint128,
    },
    AddAllowedZCaller {
        allowed_z_caller: Addr,
    },
    RemoveAllowedZCaller {
        allowed_z_caller: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Addr>)]
    GetAllowedZCallers {},
    #[returns(Config)]
    GetConfig {},
}
