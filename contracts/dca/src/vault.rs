use base::{
    pair::Pair,
    triggers::trigger::TimeInterval,
    vaults::vault::{Destination, PositionType, VaultStatus},
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal256, Timestamp, Uint128};

#[cw_serde]
pub struct Vault {
    pub id: Uint128,
    pub created_at: Timestamp,
    pub owner: Addr,
    pub label: Option<String>,
    pub destinations: Vec<Destination>,
    pub status: VaultStatus,
    pub balance: Coin,
    pub pair: Pair,
    swap_amount: Uint128,
    pub position_type: PositionType,
    pub slippage_tolerance: Option<Decimal256>,
    pub price_threshold: Option<Decimal256>,
    pub time_interval: TimeInterval,
    pub started_at: Option<Timestamp>,
}

impl Vault {
    pub fn get_swap(&self) -> Swap {
        Swap {
            address: self.pair.address.clone(),
            send_denom: self.get_swap_denom(),
            receive_denom: self.get_receive_denom(),
            amount: self.get_swap_amount().amount,
        }
    }

    fn get_swap_denom(&self) -> String {
        if self.position_type.to_owned() == PositionType::Enter {
            return self.pair.quote_denom.clone();
        }
        self.pair.base_denom.clone()
    }

    fn get_receive_denom(&self) -> String {
        if self.position_type.to_owned() == PositionType::Enter {
            return self.pair.base_denom.clone();
        }
        self.pair.quote_denom.clone()
    }

    pub fn get_swap_amount(&self) -> Coin {
        Coin {
            denom: self.get_swap_denom(),
            amount: match self.low_funds() {
                true => self.balance.amount,
                false => self.swap_amount,
            },
        }
    }

    pub fn low_funds(&self) -> bool {
        self.balance.amount < self.swap_amount
    }

    pub fn is_active(&self) -> bool {
        self.status == VaultStatus::Active
    }

    pub fn is_scheduled(&self) -> bool {
        self.status == VaultStatus::Scheduled
    }
}

pub struct VaultBuilder {
    pub created_at: Timestamp,
    pub owner: Addr,
    pub label: Option<String>,
    pub destinations: Vec<Destination>,
    pub status: VaultStatus,
    pub balance: Coin,
    pub pair: Pair,
    pub swap_amount: Uint128,
    pub position_type: PositionType,
    pub slippage_tolerance: Option<Decimal256>,
    pub price_threshold: Option<Decimal256>,
    pub time_interval: TimeInterval,
    pub started_at: Option<Timestamp>,
}

impl VaultBuilder {
    pub fn build(self, id: Uint128) -> Vault {
        Vault {
            id,
            created_at: self.created_at,
            owner: self.owner,
            label: self.label,
            destinations: self.destinations,
            status: self.status,
            balance: self.balance,
            pair: self.pair,
            swap_amount: self.swap_amount,
            position_type: self.position_type,
            slippage_tolerance: self.slippage_tolerance,
            price_threshold: self.price_threshold,
            time_interval: self.time_interval,
            started_at: self.started_at,
        }
    }
}

#[cw_serde]
pub struct Swap {
    pub address: Addr,
    pub send_denom: String,
    pub receive_denom: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct VaultDTO {
    pub id: Uint128,
    pub created_at: Timestamp,
    pub owner: Addr,
    pub label: Option<String>,
    pub destinations: Vec<Destination>,
    pub status: VaultStatus,
    pub balance: Coin,
    pub pair: Pair,
    pub swap_amount: Uint128,
    pub swap: Swap,
    pub position_type: PositionType,
    pub slippage_tolerance: Option<Decimal256>,
    pub price_threshold: Option<Decimal256>,
    pub time_interval: TimeInterval,
    pub started_at: Option<Timestamp>,
}

impl From<Vault> for VaultDTO {
    fn from(vault: Vault) -> Self {
        let swap = vault.get_swap();
        VaultDTO {
            id: vault.id,
            created_at: vault.created_at,
            owner: vault.owner.clone(),
            label: vault.label.clone(),
            destinations: vault.destinations.clone(),
            status: vault.status.clone(),
            balance: vault.balance.clone(),
            pair: vault.pair.clone(),
            position_type: vault.position_type.clone(),
            slippage_tolerance: vault.slippage_tolerance,
            swap_amount: swap.amount.clone(),
            swap,
            price_threshold: vault.price_threshold,
            time_interval: vault.time_interval,
            started_at: vault.started_at,
        }
    }
}
