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

// #[cfg(test)]
// mod get_expected_receive_amount_handler_tests {
//     use cosmwasm_std::{
//         testing::mock_dependencies, to_json_binary, Coin, ContractResult, StdError, SystemResult,
//         Uint128,
//     };

//     use astrovault::standard_pool::query_msg::SimulationResponse;

//     use crate::{
//         handlers::get_expected_receive_amount::get_expected_receive_amount_handler,
//         state::pairs::save_pair,
//         tests::constants::{DENOM_AARCH, DENOM_UUSDC},
//         types::pair::PopulatedPair,
//     };

//     #[test]
//     fn for_failed_simulation_fails() {
//         let mut deps = mock_dependencies();

//         deps.querier.update_wasm(|_| {
//             SystemResult::Ok(ContractResult::Err("simulation failed".to_string()))
//         });

//         let pair = PopulatedPair::default();

//         save_pair(deps.as_mut().storage, &pair).unwrap();

//         assert_eq!(
//             get_expected_receive_amount_handler(
//                 deps.as_ref(),
//                 Coin {
//                     denom: DENOM_AARCH.to_string(),
//                     amount: Uint128::zero()
//                 },
//                 DENOM_UUSDC.to_string()
//             )
//             .unwrap_err(),
//             StdError::GenericErr {
//                 msg: "Querier contract error: simulation failed".to_string()
//             }
//         )
//     }

//     #[test]
//     fn for_successful_simulation_returns_expected_amount() {
//         let mut deps = mock_dependencies();

//         deps.querier.update_wasm(|_| {
//             SystemResult::Ok(ContractResult::Ok(
//                 to_json_binary(&SimulationResponse {
//                     return_amount: Uint128::from(83211293u128),
//                     spread_amount: Uint128::default(),
//                     commission_amount: Uint128::from(23312u128),
//                     buybackburn_amount: Uint128::default(),
//                 })
//                 .unwrap(),
//             ))
//         });

//         let pair = PopulatedPair::default();

//         save_pair(deps.as_mut().storage, &pair).unwrap();

//         assert_eq!(
//             get_expected_receive_amount_handler(
//                 deps.as_ref(),
//                 Coin {
//                     denom: DENOM_AARCH.to_string(),
//                     amount: Uint128::zero()
//                 },
//                 DENOM_UUSDC.to_string()
//             )
//             .unwrap(),
//             Coin {
//                 denom: DENOM_UUSDC.to_string(),
//                 amount: Uint128::from(83211293u128)
//             }
//         )
//     }
// }
