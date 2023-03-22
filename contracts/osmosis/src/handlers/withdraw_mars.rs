use crate::ContractError;
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg};
use mars_helpers::msg::RedBankExecuteMsg;

pub fn withdraw_mars(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
    red_bank_address: String,
) -> Result<Response, ContractError> {
    let withdraw = RedBankExecuteMsg::Withdraw {
        denom,
        amount: None,
        recipient: None,
    };

    Ok(Response::new()
        .add_attribute("method", "withdraw_mars")
        .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: red_bank_address,
            msg: to_binary(&withdraw)?,
            funds: info.funds,
        }))))
}
