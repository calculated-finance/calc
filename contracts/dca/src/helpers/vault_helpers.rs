use crate::{
    handlers::get_events_by_resource_id::get_events_by_resource_id,
    state::swap_adjustments::get_swap_adjustment, types::vault::Vault,
};
use base::events::event::EventData;
use cosmwasm_std::{Coin, Decimal, Deps, StdResult, Uint128};

pub fn get_swap_amount(deps: &Deps, vault: Vault) -> StdResult<Coin> {
    let initial_amount = match vault.low_funds() {
        true => vault.balance.amount,
        false => vault.swap_amount,
    };

    let adjusted_amount = vault
        .clone()
        .dca_plus_config
        .map_or(initial_amount, |dca_plus_config| {
            get_swap_adjustment(deps.storage, dca_plus_config.model_id)
                .map_or(initial_amount, |adjustment_coefficient| {
                    adjustment_coefficient * initial_amount
                })
        });

    Ok(Coin {
        denom: vault.get_swap_denom(),
        amount: adjusted_amount,
    })
}

pub fn has_sufficient_funds(deps: &Deps, vault: Vault) -> StdResult<bool> {
    get_swap_amount(deps, vault).map(|swap_amount| swap_amount.amount > Uint128::new(50000))
}

pub fn get_standard_dca_received_amount(deps: Deps, vault: Vault) -> StdResult<Coin> {
    let standard_dca_received_amount = get_events_by_resource_id(deps, vault.id, None, None)?
        .events
        .iter()
        .fold(Uint128::zero(), |acc, event| match &event.data {
            EventData::DcaPlusVaultExecutionCompleted {
                sent,
                received,
                fee,
                swap_adjustment,
            } => {
                let price = Decimal::from_ratio(sent.amount, received.amount);
                let fee_percentage = Decimal::from_ratio(fee.amount, received.amount);
                let unadjusted_swap_amount = sent.amount * (Decimal::one() / swap_adjustment);
                let received_amount = unadjusted_swap_amount * (Decimal::one() / price);
                acc + received_amount * (Decimal::one() - fee_percentage)
            }
            _ => acc,
        });

    Ok(Coin {
        denom: vault.get_swap_denom(),
        amount: standard_dca_received_amount,
    })
}
