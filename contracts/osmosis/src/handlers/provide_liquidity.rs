use crate::{
    cache::{Cache, CACHE},
    contract::AFTER_PROVIDE_LIQUIDITY,
    ContractError,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, SubMsg, Uint128};
use osmosis_std::types::osmosis::gamm::v1beta1::MsgJoinSwapExternAmountIn;

pub fn provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: u64,
) -> Result<Response, ContractError> {
    CACHE.save(deps.storage, &Cache::from(pool_id.clone()))?;

    let join = MsgJoinSwapExternAmountIn {
        sender: env.contract.address.into(),
        pool_id,
        token_in: Some(info.funds[0].clone().into()),
        share_out_min_amount: Uint128::one().into(),
    };

    Ok(Response::new()
        .add_attribute("method", "provide_liquidity")
        .add_submessage(SubMsg::reply_on_success(join, AFTER_PROVIDE_LIQUIDITY)))
}
