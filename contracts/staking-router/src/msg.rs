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
    IbcDelegate {
        channel_id: String, // frontend should know about denom trace - but keep flexible for others to send in any channel id
        delegator_address: String,
        validator_address: String,
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
}
