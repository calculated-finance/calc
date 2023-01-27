use crate::{
    position_type::PositionType,
    queries::{query_base_price, query_quote_price},
};
use base::pair::Pair;
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal256, QuerierWrapper, StdResult, WasmMsg,
};
use kujira::fin::ExecuteMsg as FinExecuteMsg;

pub fn create_fin_swap_message(
    querier: QuerierWrapper,
    pair: Pair,
    swap_amount: Coin,
    slippage_tolerance: Option<Decimal256>,
) -> StdResult<CosmosMsg> {
    match slippage_tolerance {
        Some(tolerance) => {
            let position_type = match swap_amount.denom == pair.base_denom {
                true => PositionType::Exit,
                false => PositionType::Enter,
            };

            let fin_price = match position_type {
                PositionType::Enter => query_base_price(querier, pair.address.clone()),
                PositionType::Exit => query_quote_price(querier, pair.address.clone()),
            };

            let belief_price = match position_type {
                PositionType::Enter => fin_price,
                PositionType::Exit => Decimal256::one()
                    .checked_div(fin_price)
                    .expect("should return a valid inverted price for fin sell"),
            };

            create_fin_swap_with_slippage(pair.address, belief_price, tolerance, swap_amount)
        }
        None => create_fin_swap_without_slippage(pair.address, swap_amount),
    }
}

fn create_fin_swap_without_slippage(
    pair_address: Addr,
    coin_to_send_with_message: Coin,
) -> StdResult<CosmosMsg> {
    let swap_message = to_binary(&FinExecuteMsg::Swap {
        belief_price: None,
        max_spread: None,
        offer_asset: None,
        to: None,
    })?;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pair_address.to_string(),
        msg: swap_message,
        funds: vec![coin_to_send_with_message],
    }))
}

fn create_fin_swap_with_slippage(
    pair_address: Addr,
    belief_price: Decimal256,
    max_spread: Decimal256,
    coin_to_send_with_message: Coin,
) -> StdResult<CosmosMsg> {
    let swap_message = to_binary(&FinExecuteMsg::Swap {
        belief_price: Some(belief_price),
        max_spread: Some(max_spread),
        offer_asset: None,
        to: None,
    })?;

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pair_address.to_string(),
        msg: swap_message,
        funds: vec![coin_to_send_with_message],
    }))
}
