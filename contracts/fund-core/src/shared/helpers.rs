use crate::state::get_config;
use base::price_type::PriceType;
use cosmwasm_std::{Coin, Decimal, Deps, Env, StdResult, Uint128};
use std::collections::HashMap;
use swap::msg::QueryMsg;

pub fn get_allocations(deps: Deps, env: Env) -> StdResult<Vec<(String, Decimal)>> {
    let current_balances = deps
        .querier
        .query_all_balances(env.contract.address)?
        .into_iter()
        .map(|coin| (coin.denom.clone(), coin))
        .collect::<HashMap<_, _>>();

    let current_balance_values = get_current_balance_values(deps, &current_balances)?;

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
        .collect::<Vec<_>>())
}

pub fn get_current_balance_values(
    deps: Deps,
    current_balances: &HashMap<String, Coin>,
) -> StdResult<HashMap<String, Uint128>> {
    let config = get_config(deps.storage)?;

    Ok(current_balances
        .values()
        .map(|asset| {
            if asset.denom == config.base_asset {
                return (asset.denom.clone(), asset.amount);
            }

            let price: Decimal = deps
                .querier
                .query_wasm_smart(
                    config.swap.clone(),
                    &QueryMsg::GetPrice {
                        swap_amount: asset.clone(),
                        target_denom: config.base_asset.clone(),
                        price_type: PriceType::Belief,
                    },
                )
                .expect(&format!(
                    "price for swapping {:?} into {}",
                    asset, config.base_asset
                ));

            let asset_value_in_terms_of_base_denom = asset.amount * (Decimal::one() / price);

            (asset.denom.clone(), asset_value_in_terms_of_base_denom)
        })
        .collect::<HashMap<_, _>>())
}
