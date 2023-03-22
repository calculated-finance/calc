use crate::ContractError;
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg};
use mars_helpers::msg::IncentivesExecuteMsg;

pub fn claim_rewards_mars(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    incentives_address: String,
) -> Result<Response, ContractError> {
    let claim = IncentivesExecuteMsg::ClaimRewards {};

    Ok(Response::new()
        .add_attribute("method", "claim_rewards_mars")
        .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: incentives_address,
            msg: to_binary(&claim)?,
            funds: info.funds,
        }))))
}
