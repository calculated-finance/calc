use crate::state::get_config;
use cosmwasm_std::{Decimal, Deps, Env, StdResult, Uint128};
use std::collections::HashMap;
use swapper::shared::helpers::get_cheapest_swap_path;

pub fn get_allocations(deps: Deps, env: Env) -> StdResult<HashMap<String, Decimal>> {
    let config = get_config(deps.storage)?;

    let current_balances = deps
        .querier
        .query_all_balances(env.contract.address)?
        .into_iter()
        .map(|coin| (coin.denom.clone(), coin))
        .collect::<HashMap<_, _>>();

    let current_balance_values = current_balances
        .values()
        .flat_map(|asset| {
            get_cheapest_swap_path(deps, asset, &config.base_asset).map(|path| {
                (
                    asset.denom.clone(),
                    asset.amount * (Decimal::one() / path.price),
                )
            })
        })
        .collect::<HashMap<_, _>>();

    let total_fund_value = current_balance_values
        .iter()
        .map(|(_, denom_value)| denom_value)
        .sum::<Uint128>();

    Ok(current_balance_values
        .iter()
        .map(|(denom, denom_value)| {
            (
                denom.clone(),
                Decimal::from_ratio(*denom_value, total_fund_value),
            )
        })
        .collect::<HashMap<_, _>>())
}
