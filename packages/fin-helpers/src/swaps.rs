use crate::{
    position_type::PositionType,
    queries::{query_base_price, query_quote_price},
};
use base::pair::Pair;
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal256, QuerierWrapper, StdResult, SubMsg, WasmMsg,
};
use kujira::fin::ExecuteMsg as FINExecuteMsg;

pub fn create_fin_swap_message(
    querier: QuerierWrapper,
    pair: Pair,
    swap_amount: Coin,
    slippage_tolerance: Option<Decimal256>,
    reply_id: u64,
) -> StdResult<SubMsg> {
    match slippage_tolerance {
        Some(tolerance) => {
            let position_type = match swap_amount.denom == pair.quote_denom {
                true => PositionType::Enter,
                false => PositionType::Exit,
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
            Ok(create_fin_swap_with_slippage(
                pair.address.clone(),
                belief_price,
                tolerance,
                swap_amount.clone(),
                reply_id,
            ))
        }
        None => Ok(create_fin_swap_without_slippage(
            pair.address.clone(),
            swap_amount.clone(),
            reply_id,
        )),
    }
}

fn create_fin_swap_without_slippage(
    pair_address: Addr,
    coin_to_send_with_message: Coin,
    reply_id: u64,
) -> SubMsg {
    let fin_swap_msg = FINExecuteMsg::Swap {
        belief_price: None,
        max_spread: None,
        offer_asset: None,
        to: None,
    };

    let execute_message = WasmMsg::Execute {
        contract_addr: pair_address.to_string(),
        msg: to_binary(&fin_swap_msg).unwrap(),
        funds: vec![coin_to_send_with_message],
    };

    let sub_message = SubMsg {
        id: reply_id,
        msg: CosmosMsg::Wasm(execute_message),
        gas_limit: None,
        reply_on: cosmwasm_std::ReplyOn::Always,
    };

    sub_message
}

fn create_fin_swap_with_slippage(
    pair_address: Addr,
    belief_price: Decimal256,
    max_spread: Decimal256,
    coin_to_send_with_message: Coin,
    reply_id: u64,
) -> SubMsg {
    let fin_swap_msg = FINExecuteMsg::Swap {
        belief_price: Some(belief_price),
        max_spread: Some(max_spread),
        offer_asset: None,
        to: None,
    };

    let execute_message = WasmMsg::Execute {
        contract_addr: pair_address.to_string(),
        msg: to_binary(&fin_swap_msg).unwrap(),
        funds: vec![coin_to_send_with_message],
    };

    let sub_message = SubMsg {
        id: reply_id,
        msg: CosmosMsg::Wasm(execute_message),
        gas_limit: None,
        reply_on: cosmwasm_std::ReplyOn::Always,
    };

    sub_message
}
