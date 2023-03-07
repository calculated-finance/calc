use crate::{
    helpers::vault_helpers::{get_dca_plus_performance_factor, get_dca_plus_performance_fee},
    msg::DcaPlusPerformanceResponse,
    state::vaults::get_vault,
};
use base::price_type::PriceType;
use cosmwasm_std::{Coin, Deps, StdError, StdResult, Uint128};
use fin_helpers::queries::query_price;

pub fn get_dca_plus_performance_handler(
    deps: Deps,
    vault_id: Uint128,
) -> StdResult<DcaPlusPerformanceResponse> {
    let vault = get_vault(deps.storage, vault_id)?;

    let current_price = query_price(
        deps.querier,
        vault.pair.clone(),
        &Coin::new(0, vault.get_swap_denom()),
        PriceType::Belief,
    )?;

    vault.dca_plus_config.clone().map_or(
        Err(StdError::GenericErr {
            msg: format!("Vault {} is not a DCA Plus vault", vault_id),
        }),
        |_| {
            Ok(DcaPlusPerformanceResponse {
                fee: get_dca_plus_performance_fee(&vault, current_price)?,
                factor: get_dca_plus_performance_factor(&vault, current_price)?,
            })
        },
    )
}
