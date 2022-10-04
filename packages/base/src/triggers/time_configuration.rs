use cosmwasm_std::{Addr, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::trigger::{Trigger, TriggerBuilder, TriggerVariant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeConfiguration {
    pub time_interval: TimeInterval,
    pub triggers_remaining: u16,
    pub target_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimeInterval {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl TimeConfiguration {
    pub fn is_final_trigger(&self) -> bool {
        self.triggers_remaining == 1
    }
}

impl From<TimeConfiguration> for TriggerBuilder<TimeConfiguration> {
    fn from(time_configuration: TimeConfiguration) -> Self {
        TriggerBuilder {
            id: Uint128::zero(),
            owner: Addr::unchecked(""),
            variant: TriggerVariant::Time,
            vault_id: Uint128::zero(),
            configuration: time_configuration,
        }
    }
}

impl TriggerBuilder<TimeConfiguration> {
    pub fn new_time_trigger() -> TriggerBuilder<TimeConfiguration> {
        let time_trigger_configuration: TimeConfiguration = TimeConfiguration {
            time_interval: TimeInterval::Daily,
            triggers_remaining: 0,
            target_time: Timestamp::default(),
        };
        TriggerBuilder {
            id: Uint128::zero(),
            owner: Addr::unchecked(""),
            variant: TriggerVariant::Time,
            vault_id: Uint128::zero(),
            configuration: time_trigger_configuration,
        }
    }

    pub fn id(mut self, id: Uint128) -> TriggerBuilder<TimeConfiguration> {
        self.id = id;
        self
    }

    pub fn owner(mut self, owner: Addr) -> TriggerBuilder<TimeConfiguration> {
        self.owner = owner;
        self
    }

    pub fn vault_id(mut self, vault_id: Uint128) -> TriggerBuilder<TimeConfiguration> {
        self.vault_id = vault_id;
        self
    }

    pub fn time_interval(
        mut self,
        time_interval: TimeInterval,
    ) -> TriggerBuilder<TimeConfiguration> {
        self.configuration.time_interval = time_interval;
        self
    }

    pub fn triggers_remaining(
        mut self,
        triggers_remaining: u16,
    ) -> TriggerBuilder<TimeConfiguration> {
        self.configuration.triggers_remaining = triggers_remaining;
        self
    }

    pub fn target_time(mut self, target_time: Timestamp) -> TriggerBuilder<TimeConfiguration> {
        self.configuration.target_time = target_time;
        self
    }

    pub fn build(self) -> Trigger<TimeConfiguration> {
        Trigger {
            id: self.id,
            owner: self.owner,
            variant: self.variant,
            vault_id: self.vault_id,
            configuration: self.configuration,
        }
    }
}
