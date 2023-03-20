use cosmwasm_std::{Coin, Deps, Env, StdResult, Uint128};
use osmosis_std::types::osmosis::gamm::v1beta1::{
    QuerySwapExactAmountInRequest, QuerySwapExactAmountInResponse, SwapAmountInRoute,
};

pub fn get_price(
    deps: Deps,
    env: Env,
    pool_id: u64,
    denom_in: String,
    amount_in: Uint128,
    denom_out: String,
) -> StdResult<QuerySwapExactAmountInResponse> {
    QuerySwapExactAmountInRequest {
        sender: env.contract.address.into(),
        pool_id,
        routes: vec![SwapAmountInRoute {
            pool_id,
            token_out_denom: denom_out,
        }],
        token_in: Coin::new(amount_in.into(), denom_in).to_string(),
    }
    .query(&deps.querier)
}
