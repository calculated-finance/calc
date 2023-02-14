use crate::state::cache::MIGRATION_CACHE;
use base::ContractError;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Reply, Response, SubMsg, WasmMsg::Execute as WasmExecuteMsg,
};
use fund_router::msg::ExecuteMsg as RouterExecuteMsg;

pub fn assign_new_fund_to_router(deps: DepsMut, _reply: Reply) -> Result<Response, ContractError> {
    let cache = MIGRATION_CACHE.load(deps.storage)?;

    let new_fund_address = cache
        .new_fund_address
        .expect("address should exist from previous logic");

    let assign_fund_msg = SubMsg::new(CosmosMsg::Wasm(WasmExecuteMsg {
        contract_addr: cache.router_address.to_string(),
        funds: vec![],
        msg: to_binary(&RouterExecuteMsg::AssignFund {
            fund_address: new_fund_address.clone(),
        })?,
    }));

    Ok(Response::new()
        .add_attribute("method", "assign_new_fund_to_router")
        .add_attribute("new_fund_address", new_fund_address)
        .add_submessage(assign_fund_msg))
}
