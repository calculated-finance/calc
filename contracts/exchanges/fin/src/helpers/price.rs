use std::str::FromStr;

use cosmwasm_std::{Coin, Decimal, QuerierWrapper, StdError, StdResult, Uint128};
use kujira_fin::{ConfigResponse, QueryMsg};
use kujira_std::{Precise, Precision};

use crate::types::{pair::Pair, position_type::PositionType};

pub fn get_target_price(
    querier: &QuerierWrapper,
    swap_amount: Coin,
    pair: &Pair,
    target_receive_amount: Uint128,
) -> StdResult<Decimal> {
    let pair_config =
        querier.query_wasm_smart::<ConfigResponse>(pair.address.clone(), &QueryMsg::Config {})?;

    let decimal_delta = pair_config.decimal_delta;

    if decimal_delta < 0 {
        return Err(StdError::GenericErr {
            msg: "Negative decimal deltas are not supported".to_string(),
        });
    }

    calculate_target_price(
        swap_amount,
        pair,
        target_receive_amount,
        decimal_delta,
        pair_config.price_precision,
    )
}

fn calculate_target_price(
    swap_amount: Coin,
    pair: &Pair,
    target_receive_amount: Uint128,
    decimal_delta: i8,
    precision: Precision,
) -> StdResult<Decimal> {
    let target_price = match pair.position_type(swap_amount.denom.clone()) {
        PositionType::Enter => Decimal::from_ratio(swap_amount.amount, target_receive_amount),
        PositionType::Exit => Decimal::from_ratio(target_receive_amount, swap_amount.amount),
    };

    if decimal_delta == 0 {
        return Ok(target_price.round(&precision));
    }

    let adjustment =
        Decimal::from_str(&10u128.pow(decimal_delta.unsigned_abs() as u32).to_string()).unwrap();

    let rounded_price = target_price
        .checked_mul(adjustment)
        .unwrap()
        .round(&precision);

    Ok(rounded_price.checked_div(adjustment).unwrap())
}

#[cfg(test)]
mod calculate_target_price_tests {
    use super::*;

    #[test]
    fn should_be_correct_when_buying_on_fin() {
        let pair = Pair::default();

        assert_eq!(
            calculate_target_price(
                Coin::new(100, pair.base_denom.clone()),
                &pair,
                Uint128::new(20),
                0,
                Precision::DecimalPlaces(3)
            )
            .unwrap()
            .to_string(),
            "5"
        );
    }

    #[test]
    fn should_be_correct_when_selling_on_fin() {
        let pair = Pair::default();

        assert_eq!(
            calculate_target_price(
                Coin::new(100, pair.base_denom.clone()),
                &pair,
                Uint128::new(20),
                0,
                Precision::DecimalPlaces(3)
            )
            .unwrap()
            .to_string(),
            "0.2"
        );
    }

    #[test]
    fn should_truncate_price_to_three_decimal_places() {
        let pair = Pair::default();

        assert_eq!(
            calculate_target_price(
                Coin::new(100, pair.base_denom.clone()),
                &pair,
                Uint128::new(10),
                0,
                Precision::DecimalPlaces(3)
            )
            .unwrap()
            .to_string(),
            "0.333"
        );
    }

    #[test]
    fn for_fin_buy_with_decimal_delta_should_truncate() {
        let pair = Pair::default();
        let swap_amount = Coin::new(1000000, pair.base_denom.clone());
        let target_receive_amount = Uint128::new(747943156999999);
        let decimal_delta = 12;
        let precision = Precision::DecimalPlaces(2);

        assert_eq!(
            Decimal::from_ratio(swap_amount.amount, target_receive_amount).to_string(),
            "0.000000001336999998"
        );
        assert_eq!(
            calculate_target_price(
                swap_amount,
                &pair,
                target_receive_amount,
                decimal_delta,
                precision
            )
            .unwrap()
            .to_string(),
            "0.00000000133699"
        );
    }

    #[test]
    fn for_fin_sell_with_decimal_delta_should_truncate() {
        let pair = Pair::default();
        let swap_amount = Coin::new(747943156999999, pair.quote_denom.clone());
        let target_receive_amount = Uint128::new(1000000);
        let decimal_delta = 12;
        let precision = Precision::DecimalPlaces(2);

        assert_eq!(
            Decimal::from_ratio(target_receive_amount, swap_amount.amount).to_string(),
            "0.000000001336999998"
        );
        assert_eq!(
            calculate_target_price(
                swap_amount,
                &pair,
                target_receive_amount,
                decimal_delta,
                precision
            )
            .unwrap()
            .to_string(),
            "0.00000000133699"
        );
    }
}
