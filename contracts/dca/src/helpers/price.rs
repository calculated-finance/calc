use super::orders::FinOrderResponseWithoutDenom;
use crate::types::pair::Pair;
use cosmwasm_std::{Addr, Coin, Decimal, QuerierWrapper, StdResult, Uint128};
use kujira::{fin::QueryMsg as FinQueryMsg, Asset, AssetInfo, Denom};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinSimulationResponse {
    pub return_amount: Uint128,
}

pub fn query_belief_price(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_denom: &str,
) -> StdResult<Decimal> {
    query_price(querier, pair, &Coin::new(1000000, swap_denom))
}

pub fn query_price(
    querier: &QuerierWrapper,
    pair: &Pair,
    swap_amount: &Coin,
) -> StdResult<Decimal> {
    let simulation = querier.query_wasm_smart::<FinSimulationResponse>(
        pair.address.clone(),
        &FinQueryMsg::Simulation {
            offer_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: Denom::from(swap_amount.denom.clone()),
                },
                amount: swap_amount.amount,
            },
        },
    )?;

    Ok(Decimal::from_ratio(
        swap_amount.amount,
        simulation.return_amount,
    ))
}

pub fn calculate_slippage(actual_price: Decimal, belief_price: Decimal) -> Decimal {
    let difference = actual_price
        .checked_sub(belief_price)
        .unwrap_or(Decimal::zero());

    if difference.is_zero() {
        return Decimal::zero();
    }

    difference / belief_price
}

pub fn query_order_details(
    querier: QuerierWrapper,
    pair_address: Addr,
    order_idx: Uint128,
) -> StdResult<FinOrderResponseWithoutDenom> {
    let fin_order_query_msg = FinQueryMsg::Order { order_idx };
    Ok(querier.query_wasm_smart(pair_address, &fin_order_query_msg)?)
}
