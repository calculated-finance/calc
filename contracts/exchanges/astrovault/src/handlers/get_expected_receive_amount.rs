use crate::state::config::get_config;
use astrovault::router::query_msg::{QueryMsg, QueryRouteSwapSimulation};
use astrovault::router::state::HopV2;
use cosmwasm_std::{from_json, Binary, Coin, Deps, StdError, StdResult};

pub fn get_expected_receive_amount_handler(
    deps: Deps,
    swap_amount: Coin,
    target_denom: String,
    route: &Binary,
) -> StdResult<Coin> {
    let hops = from_json::<Vec<HopV2>>(route)?;

    let config = get_config(deps.storage)?;

    let response = deps.querier.query_wasm_smart::<QueryRouteSwapSimulation>(
        config.router_address,
        &QueryMsg::RouteSwapSimulation {
            amount: swap_amount.amount,
            hops,
        },
    )?;

    if response.to.info.to_string() != target_denom {
        return Err(StdError::generic_err("Invalid return denom"));
    }

    Ok(Coin {
        denom: response.to.info.to_string(),
        amount: response.to.amount,
    })
}
