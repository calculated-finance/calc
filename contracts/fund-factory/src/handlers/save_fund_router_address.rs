use base::{helpers::message_helpers::get_attribute_in_event, ContractError};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Reply, Response, SubMsg,
    WasmMsg::Instantiate as WasmInstantiate,
};

use crate::{
    contract::AFTER_INSTANTIATE_FUND_CORE_REPLY_ID,
    state::{
        cache::{Cache, CACHE},
        config::get_config,
        fund_routers::save_fund_router,
    },
};
use fund_core::msg::InstantiateMsg as CoreInstantiateMsg;

pub fn save_fund_router_address(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;

    let instantiate_fund_router_response = reply.result.unwrap();

    let fund_router_address = Addr::unchecked(get_attribute_in_event(
        &instantiate_fund_router_response.events,
        "instantiate",
        "_contract_address",
    )?);

    save_fund_router(
        deps.storage,
        cache.owner.clone(),
        fund_router_address.clone(),
    )?;

    CACHE.save(
        deps.storage,
        &Cache {
            owner: cache.owner,
            fund_router_address: Some(fund_router_address.clone()),
        },
    )?;

    let config = get_config(deps.storage)?;

    let fund_core_instantiate_msg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmInstantiate {
            admin: None,
            label: format!("CALC-MF-CORE"),
            code_id: config.fund_core_code_id,
            funds: vec![],
            msg: to_binary(&CoreInstantiateMsg {})?,
        }),
        AFTER_INSTANTIATE_FUND_CORE_REPLY_ID,
    );

    Ok(Response::new()
        .add_attribute("method", "save_fund_router_address")
        .add_attribute("fund_router_address", fund_router_address)
        .add_submessage(fund_core_instantiate_msg))
}
