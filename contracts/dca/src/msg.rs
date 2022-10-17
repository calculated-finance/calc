use base::events::event::Event;
use cosmwasm_std::{Decimal256, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use base::pair::Pair;
use base::triggers::trigger::{TimeInterval, Trigger};
use base::vaults::vault::PositionType;

use crate::vault::Vault;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub fee_collector: String,
    pub fee_percent: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub admin: String,
    pub fee_collector: String,
    pub fee_percent: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePair {
        address: String,
        base_denom: String,
        quote_denom: String,
    },
    DeletePair {
        address: String,
    },
    CreateVault {
        pair_address: String,
        position_type: PositionType,
        slippage_tolerance: Option<Decimal256>,
        swap_amount: Uint128,
        time_interval: TimeInterval,
        target_start_time_utc_seconds: Option<Uint64>,
        target_price: Option<Decimal256>,
    },
    Deposit {
        vault_id: Uint128,
    },
    CancelVault {
        address: String,
        vault_id: Uint128,
    },
    ExecuteTrigger {
        trigger_id: Uint128,
    },
    UpdateConfig {
        fee_collector: Option<String>,
        fee_percent: Option<Uint128>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPairs {},
    GetTimeTriggers {},
    GetVault {
        vault_id: Uint128,
    },
    GetVaultsByAddress {
        address: String,
    },
    GetEventsByResourceId {
        resource_id: Uint128,
    },
    GetEvents {
        start_after: Option<u64>,
        limit: Option<u8>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairsResponse {
    pub pairs: Vec<Pair>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TriggersResponse {
    pub triggers: Vec<Trigger>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TriggerIdsResponse {
    pub trigger_ids: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultResponse {
    pub vault: Vault,
    pub trigger: Option<Trigger>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VaultsResponse {
    pub vaults: Vec<Vault>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EventsResponse {
    pub events: Vec<Event>,
}
