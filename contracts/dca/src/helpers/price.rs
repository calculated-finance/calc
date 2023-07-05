use cosmwasm_std::{Addr, Coin, Decimal, QuerierWrapper, StdResult, Storage, Uint128};
use exchange::msg::QueryMsg;

use crate::state::config::get_config;

pub fn get_twap_to_now(
    querier: &QuerierWrapper,
    exchange_contract_address: Addr,
    swap_denom: String,
    target_denom: String,
    period: u64,
) -> StdResult<Decimal> {
    querier.query_wasm_smart::<Decimal>(
        exchange_contract_address,
        &QueryMsg::GetTwapToNow {
            swap_denom,
            target_denom,
            period,
        },
    )
}

pub fn get_expected_receive_amount(
    querier: &QuerierWrapper,
    exchange_contract_address: Addr,
    swap_amount: Coin,
    target_denom: String,
) -> StdResult<Uint128> {
    Ok(querier
        .query_wasm_smart::<Coin>(
            exchange_contract_address,
            &QueryMsg::GetExpectedReceiveAmount {
                swap_amount,
                target_denom,
            },
        )?
        .amount)
}

pub fn get_slippage(
    querier: &QuerierWrapper,
    exchange_contract_address: Addr,
    swap_amount: Coin,
    target_denom: String,
    beleif_price: Decimal,
) -> StdResult<Decimal> {
    let expected_receive_amount = get_expected_receive_amount(
        querier,
        exchange_contract_address,
        swap_amount.clone(),
        target_denom,
    );

    if expected_receive_amount.is_err() {
        return Ok(Decimal::percent(100));
    }

    let expected_price = Decimal::from_ratio(swap_amount.amount, expected_receive_amount?);
    let price_diff = expected_price - beleif_price;

    Ok(price_diff / beleif_price)
}

pub fn get_price(
    querier: &QuerierWrapper,
    storage: &dyn Storage,
    exchange_contract_address: Addr,
    swap_amount: Coin,
    target_denom: String,
) -> StdResult<Decimal> {
    let expected_receive_amount = get_expected_receive_amount(
        querier,
        exchange_contract_address,
        swap_amount.clone(),
        target_denom.clone(),
    );

    if expected_receive_amount.is_err() {
        let config = get_config(storage)?;

        return get_twap_to_now(
            querier,
            config.exchange_contract_address,
            swap_amount.denom,
            target_denom,
            config.twap_period,
        );
    }

    Ok(Decimal::from_ratio(
        swap_amount.amount,
        expected_receive_amount?,
    ))
}
