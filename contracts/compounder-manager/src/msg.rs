use cosmwasm_std::{Addr, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Delegate {
        delegator_address: Addr,
        validator_address: Addr,
    },
    DelegateOnBehalf {
        delegator_address: Addr,
        validator_address: Addr
    },
    Undelegate {
        delegator_address: Addr,
        validator_address: Addr,
        denom: String,
        amount: Option<Uint128>,
    },
    Withdraw {
        delegator_address: Addr,
        to_address: Addr,
    },
    SetCompounderCodeId {
        code_id: Uint64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetCompounders {},
    GetBalances { delegator_address: Addr },
    GetDelegations { delegator_address: Addr },
    GetUnbondingDelegations {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CompoundersResponse {
    pub compounders: Vec<Addr>,
}
