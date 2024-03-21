use cosmwasm_std::{Binary, Coin, Decimal256, Deps, StdResult};

use super::get_expected_receive_amount::get_expected_receive_amount_handler;

pub const AMOUNT_TO_SIMULATE_TWAP: u128 = 1_000_000u128;
pub const FALLBACK_AMOUNT_TO_SIMULATE_TWAP: u128 = 1_000_000_000_000_000_000u128;

pub fn get_twap_to_now_handler(
    deps: Deps,
    swap_denom: String,
    target_denom: String,
    route: Binary,
) -> StdResult<Decimal256> {
    let response = get_expected_receive_amount_handler(
        deps,
        Coin {
            amount: AMOUNT_TO_SIMULATE_TWAP.into(),
            denom: swap_denom.clone(),
        },
        target_denom.clone(),
        route.clone(),
    )?;

    if response.amount.is_zero() {
        let response = get_expected_receive_amount_handler(
            deps,
            Coin {
                amount: FALLBACK_AMOUNT_TO_SIMULATE_TWAP.into(),
                denom: swap_denom,
            },
            target_denom.clone(),
            route,
        )?;

        if response.amount.is_zero() {
            return Ok(Decimal256::zero());
        }

        return Ok(Decimal256::from_ratio(
            FALLBACK_AMOUNT_TO_SIMULATE_TWAP,
            response.amount.u128(),
        ));
    }

    Ok(Decimal256::from_ratio(
        AMOUNT_TO_SIMULATE_TWAP,
        response.amount.u128(),
    ))
}
