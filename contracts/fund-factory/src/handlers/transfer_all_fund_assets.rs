use crate::{
    contract::AFTER_TRANSFER_ALL_FUND_ASSETS_REPLY_ID,
    state::cache::{MigrationCache, MIGRATION_CACHE},
};
use base::{helpers::message_helpers::get_attribute_in_event, ContractError};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Reply, Response, SubMsg,
    WasmMsg::Execute as WasmExecuteMsg,
};
use fund_core::msg::ExecuteMsg as FundExecuteMsg;

pub fn transfer_all_fund_assets(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let cache = MIGRATION_CACHE.load(deps.storage)?;

    let instantiate_fund_response = reply.result.unwrap();

    let new_fund_address = Addr::unchecked(get_attribute_in_event(
        &instantiate_fund_response.events,
        "instantiate",
        "_contract_address",
    )?);

    MIGRATION_CACHE.save(
        deps.storage,
        &MigrationCache {
            new_fund_address: Some(new_fund_address.clone()),
            ..cache.clone()
        },
    )?;

    let migrate_fund_msg = SubMsg::reply_always(
        CosmosMsg::Wasm(WasmExecuteMsg {
            contract_addr: cache.old_fund_address.to_string(),
            funds: vec![],
            msg: to_binary(&FundExecuteMsg::Migrate {
                new_fund_address: new_fund_address.clone(),
            })?,
        }),
        AFTER_TRANSFER_ALL_FUND_ASSETS_REPLY_ID,
    );

    Ok(Response::new()
        .add_attribute("method", "transfer_all_fund_assets")
        .add_attribute("old_fund_address", cache.old_fund_address)
        .add_attribute("new_fund_address", new_fund_address)
        .add_submessage(migrate_fund_msg))
}
