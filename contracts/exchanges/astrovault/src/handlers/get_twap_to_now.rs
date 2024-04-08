use cosmwasm_std::{Binary, Coin, Decimal256, Deps, StdError, StdResult};

use super::get_expected_receive_amount::get_expected_receive_amount_handler;

pub const AMOUNT_TO_SIMULATE_TWAP: u128 = 1_000_000u128;
pub const FALLBACK_AMOUNT_TO_SIMULATE_TWAP: u128 = 1_000_000_000_000_000_000u128;

pub fn get_twap_to_now_handler(
    deps: Deps,
    swap_denom: String,
    target_denom: String,
    period: u64,
    route: &Binary,
) -> StdResult<Decimal256> {
    if period != 0 {
        return Err(StdError::generic_err(format!(
            "Cannot get twap for period of {} seconds, only 0 is supported",
            period
        )));
    }

    let coin = get_expected_receive_amount_handler(
        deps,
        Coin {
            denom: swap_denom.clone(),
            amount: AMOUNT_TO_SIMULATE_TWAP.into(),
        },
        target_denom.clone(),
        route,
    )?;

    if coin.amount.is_zero() {
        let coin = get_expected_receive_amount_handler(
            deps,
            Coin {
                denom: swap_denom,
                amount: FALLBACK_AMOUNT_TO_SIMULATE_TWAP.into(),
            },
            target_denom,
            route,
        )?;

        return Ok(Decimal256::from_ratio(
            FALLBACK_AMOUNT_TO_SIMULATE_TWAP,
            coin.amount.u128(),
        ));
    }

    Ok(Decimal256::from_ratio(
        AMOUNT_TO_SIMULATE_TWAP,
        coin.amount.u128(),
    ))
}

#[cfg(test)]
mod get_twap_to_now_tests {
    use astrovault::{assets::pools::PoolInfoInput, router::state::HopV2};
    use cosmwasm_std::{testing::mock_dependencies, to_json_binary, StdError};

    use crate::{
        handlers::get_twap_to_now::get_twap_to_now_handler,
        tests::constants::{DENOM_AARCH, DENOM_UUSDC},
    };

    #[test]
    fn with_period_larger_than_zero_fails() {
        assert_eq!(
            get_twap_to_now_handler(
                mock_dependencies().as_ref(),
                DENOM_AARCH.to_string(),
                DENOM_UUSDC.to_string(),
                10,
                &to_json_binary(&vec![HopV2::StandardHopInfo {
                    pool: PoolInfoInput::Id("test".to_string()),
                    from_asset_index: 1
                }])
                .unwrap()
            )
            .unwrap_err(),
            StdError::generic_err("Cannot get twap for period of 10 seconds, only 0 is supported")
        )
    }
}
