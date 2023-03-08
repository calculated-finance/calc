use crate::{
    error::ContractError,
    helpers::{
        disbursement_helpers::{get_disbursement_messages, get_fee_messages},
        vault_helpers::get_dca_plus_performance_fee,
    },
    state::vaults::{get_vault, update_vault},
};
use base::{price_type::PriceType, vaults::vault::VaultStatus};
use cosmwasm_std::{Coin, DepsMut, Env, Response, Uint128};
use fin_helpers::queries::query_price;

pub fn claim_escrowed_funds_handler(
    deps: DepsMut,
    env: Env,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    let mut vault = get_vault(deps.storage, vault_id)?;

    if [VaultStatus::Active, VaultStatus::Scheduled].contains(&vault.status) {
        return Ok(Response::new());
    }

    vault.dca_plus_config.clone().map_or(
        Err(ContractError::CustomError {
            val: "Vault is not a DCA+ vault".to_string(),
        }),
        |mut dca_plus_config| {
            let current_price = query_price(
                deps.querier,
                vault.pair.clone(),
                &Coin {
                    denom: vault.get_swap_denom(),
                    amount: Uint128::one(),
                },
                PriceType::Belief,
            )?;

            let performance_fee = get_dca_plus_performance_fee(&vault, current_price)?;
            let amount_to_disburse = dca_plus_config
                .escrowed_balance
                .checked_sub(performance_fee.amount)
                .expect("the escrowed amount to return should be at least 0");

            dca_plus_config.escrowed_balance = Uint128::zero();
            vault.dca_plus_config = Some(dca_plus_config);
            update_vault(deps.storage, &vault)?;

            Ok(Response::new()
                .add_submessages(get_disbursement_messages(
                    deps.as_ref(),
                    &vault,
                    amount_to_disburse,
                )?)
                .add_submessages(get_fee_messages(
                    deps.as_ref(),
                    env,
                    vec![performance_fee.amount],
                    vault.get_receive_denom(),
                )?))
        },
    )
}
