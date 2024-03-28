use astroport::router::{QueryMsg, SimulateSwapOperationsResponse, SwapOperation};
use cosmwasm_std::{from_json, Binary, Coin, Deps, StdResult};

use crate::state::config::get_config;

pub fn get_expected_receive_amount_handler(
    deps: Deps,
    swap_amount: Coin,
    target_denom: String,
    route: Binary,
) -> StdResult<Coin> {
    let config = get_config(deps.storage)?;

    let swap_operations = from_json::<Vec<SwapOperation>>(&route)?;

    let response = deps
        .querier
        .query_wasm_smart::<SimulateSwapOperationsResponse>(
            config.router_address,
            &QueryMsg::SimulateSwapOperations {
                offer_amount: swap_amount.amount,
                operations: swap_operations,
            },
        )?;

    Ok(Coin {
        denom: target_denom,
        amount: response.amount,
    })
}
