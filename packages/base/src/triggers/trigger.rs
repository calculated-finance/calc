use std::fmt::{Display, Formatter, Result};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal256, Timestamp, Uint128};
use enum_as_inner::EnumAsInner;

#[cw_serde]
pub enum TimeInterval {
    HalfHourly,
    Hourly,
    HalfDaily,
    Daily,
    Weekly,
    Fortnightly,
    Monthly,
}

impl Display for TimeInterval {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl From<String> for TimeInterval {
    fn from(s: String) -> Self {
        match s.as_str() {
            "HalfHourly" => TimeInterval::HalfHourly,
            "Hourly" => TimeInterval::Hourly,
            "HalfDaily" => TimeInterval::HalfDaily,
            "Daily" => TimeInterval::Daily,
            "Weekly" => TimeInterval::Weekly,
            "Fortnightly" => TimeInterval::Fortnightly,
            "Monthly" => TimeInterval::Monthly,
            _ => panic!("Invalid time interval"),
        }
    }
}

#[derive(EnumAsInner)]
#[cw_serde]
pub enum TriggerConfiguration {
    Time {
        target_time: Timestamp,
    },
    FINLimitOrder {
        target_price: Decimal256,
        order_idx: Option<Uint128>,
    },
}

#[cw_serde]
pub struct Trigger {
    pub vault_id: Uint128,
    pub configuration: TriggerConfiguration,
}
