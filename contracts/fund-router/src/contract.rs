use base::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
};
use kujira::denom::Denom;
use kujira::msg::{DenomMsg, KujiraMsg};

use crate::handlers::assign_fund_core::assign_fund_core;
use crate::handlers::get_fund_core::get_fund_core;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::fund_cores::initialise_fund_cores;

pub const AFTER_INSTANTIATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<KujiraMsg>, ContractError> {
    initialise_fund_cores(deps)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_submessage(SubMsg::new(DenomMsg::Create {
            subdenom: Denom::from(msg.token_name),
        })))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AssignFundCore { fund_core_address } => {
            assign_fund_core(deps, fund_core_address)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFundCore {} => to_binary(&get_fund_core(deps)?),
    }
}
