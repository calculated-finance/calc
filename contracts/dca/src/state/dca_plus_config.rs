use crate::types::{
    dca_plus_config::DcaPlusConfig, performance_assessment_strategy::PerformanceAssessmentStrategy,
    swap_adjustment_strategy::SwapAdjustmentStrategy, vault::Vault,
};
use cosmwasm_std::{Storage, Uint128};
use cw_storage_plus::Map;

pub const DCA_PLUS_CONFIGS: Map<u128, DcaPlusConfig> = Map::new("dca_plus_configs_v20");

pub fn get_dca_plus_config(store: &dyn Storage, vault_id: Uint128) -> Option<DcaPlusConfig> {
    DCA_PLUS_CONFIGS
        .may_load(store, vault_id.into())
        .unwrap_or(None)
}

pub fn dca_plus_config_from(vault: &Vault) -> Option<DcaPlusConfig> {
    match vault.swap_adjustment_strategy.clone() {
        Some(SwapAdjustmentStrategy::RiskWeightedAverage { model_id, .. }) => {
            match vault.performance_assessment_strategy.clone() {
                Some(PerformanceAssessmentStrategy::CompareToStandardDca {
                    swapped_amount,
                    received_amount,
                }) => Some(DcaPlusConfig {
                    escrow_level: vault.escrow_level,
                    model_id,
                    total_deposit: vault.deposited_amount.clone(),
                    standard_dca_swapped_amount: swapped_amount,
                    standard_dca_received_amount: received_amount,
                    escrowed_balance: vault.escrowed_amount.clone(),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}
