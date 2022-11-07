use base::{
    pair::Pair,
    triggers::trigger::TimeInterval,
    vaults::vault::{Destination, PositionType, VaultStatus},
};
use cosmwasm_std::{coin, Addr, Coin, Decimal, Timestamp, Uint128};

use super::vault::Vault;

pub struct VaultBuilder {
    pub created_at: Timestamp,
    pub owner: Addr,
    pub label: Option<String>,
    pub destinations: Vec<Destination>,
    pub status: VaultStatus,
    pub balance: Coin,
    pub pair: Pair,
    pub swap_amount: Uint128,
    pub position_type: Option<PositionType>,
    pub slippage_tolerance: Option<Decimal>,
    pub minimum_receive_amount: Option<Uint128>,
    pub time_interval: TimeInterval,
    pub started_at: Option<Timestamp>,
}

impl VaultBuilder {
    pub fn new(
        created_at: Timestamp,
        owner: Addr,
        label: Option<String>,
        destinations: Vec<Destination>,
        status: VaultStatus,
        balance: Coin,
        pair: Pair,
        swap_amount: Uint128,
        position_type: Option<PositionType>,
        slippage_tolerance: Option<Decimal>,
        minimum_receive_amount: Option<Uint128>,
        time_interval: TimeInterval,
        started_at: Option<Timestamp>,
    ) -> VaultBuilder {
        VaultBuilder {
            created_at,
            owner,
            label,
            destinations,
            status,
            balance,
            pair,
            swap_amount,
            position_type,
            slippage_tolerance,
            minimum_receive_amount,
            time_interval,
            started_at,
        }
    }

    pub fn build(self, id: Uint128) -> Vault {
        Vault {
            id,
            created_at: self.created_at,
            owner: self.owner,
            label: self.label,
            destinations: self.destinations,
            status: self.status,
            balance: self.balance.clone(),
            pair: self.pair.clone(),
            swap_amount: self.swap_amount,
            slippage_tolerance: self.slippage_tolerance,
            minimum_receive_amount: self.minimum_receive_amount,
            time_interval: self.time_interval,
            started_at: self.started_at,
            swapped_amount: coin(0, self.balance.denom.clone()),
            received_amount: coin(
                0,
                match self.balance.denom == self.pair.quote_denom {
                    true => self.pair.base_denom,
                    false => self.pair.quote_denom,
                },
            ),
            trigger: None,
        }
    }
}
