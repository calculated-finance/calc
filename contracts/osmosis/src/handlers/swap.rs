use crate::ContractError;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cosmwasm_std::{SubMsg, Uint128};
use osmosis_std::types::osmosis::gamm::v1beta1::{MsgSwapExactAmountIn, SwapAmountInRoute};

pub fn swap(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: u64,
    denom_out: String,
) -> Result<Response, ContractError> {
    let slippage = Uint128::one();

    let swap = MsgSwapExactAmountIn {
        sender: env.contract.address.into(),
        token_in: Some(info.funds[0].clone().into()),
        token_out_min_amount: slippage.into(),
        routes: vec![SwapAmountInRoute {
            pool_id,
            token_out_denom: denom_out,
        }],
    };

    Ok(Response::new()
        .add_attribute("method", "swap")
        .add_submessage(SubMsg::new(swap)))
}
