use crate::ContractError;
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg};
use mars_helpers::msg::RedBankExecuteMsg;

pub fn deposit_mars(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    red_bank_address: String,
) -> Result<Response, ContractError> {
    let deposit = RedBankExecuteMsg::Deposit { on_behalf_of: None };

    Ok(Response::new()
        .add_attribute("method", "deposit_mars")
        .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: red_bank_address,
            msg: to_binary(&deposit)?,
            funds: info.funds,
        }))))
}
