use cosmwasm_std::{QuerierWrapper, Coin, Decimal256, ReplyOn, StdResult, SubMsg, Uint128, to_binary};
use osmosis_std::types::osmosis::gamm::v1beta1::{MsgSwapExactAmountIn, SwapAmountInRoute};

use crate::pool::Pool;



pub fn create_osmosis_swap_message(
    querier: QuerierWrapper,
    sender: String,
    pool: Pool,
    swap_amount: Coin,
    slippage_tolerance: Option<Decimal256>,
    reply_id: Option<u64>,
    reply_on: Option<ReplyOn>,
) -> StdResult<SubMsg> {
    // let belief_price = slippage_tolerance
    //     .map(|_| query_belief_price(querier, &pair, &swap_amount.denom).expect("belief price"));

    let slippage = Uint128::one();

    let token_out_denom = if swap_amount.denom == pool.base_denom {
        pool.quote_denom
    } else {
        pool.base_denom
    };

    let swap = MsgSwapExactAmountIn {
        sender,
        token_in: Some(swap_amount.into()),
        token_out_min_amount: slippage.into(),
        routes: vec![SwapAmountInRoute {
            pool_id: pool.pool_id,
            token_out_denom
        }],
    };

    // let swap_message = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: pair.address.to_string(),
    //     msg: to_binary(&ExecuteMsg::Swap {
    //         belief_price: belief_price
    //             .map(|price| Decimal256::from_str(&price.to_string()).unwrap()),
    //         max_spread: slippage_tolerance.clone(),
    //         to: None,
    //         offer_asset: None,
    //     })?,
    //     funds: vec![swap_amount],
    // });

    Ok(SubMsg::reply_always(swap, reply_id.unwrap_or(0)))
}
