use base::ContractError;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, MessageInfo, Response, SubMsg,
    WasmMsg::Instantiate as WasmInstantiate,
};

use crate::{
    contract::AFTER_INSTANTIATE_FUND_FOR_MIGRATION_REPLY_ID,
    state::{
        cache::{MigrationCache, MIGRATION_CACHE},
        config::get_config,
    },
    validation_helpers::assert_sender_is_owner_or_admin,
};

use fund_core::msg::InstantiateMsg as FundInstantiateMsg;
use fund_router::msg::{ConfigResponse, QueryMsg as RouterQueryMsg};

pub fn migrate_to_latest_code_id(
    deps: DepsMut,
    info: MessageInfo,
    router: Addr,
) -> Result<Response, ContractError> {
    let router_config: ConfigResponse = deps
        .querier
        .query_wasm_smart(router.clone(), &RouterQueryMsg::GetConfig {})?;

    assert_sender_is_owner_or_admin(deps.storage, info.sender, &router_config)?;

    MIGRATION_CACHE.save(
        deps.storage,
        &MigrationCache {
            router_address: router,
        },
    )?;

    let config = get_config(deps.storage)?;

    let fund_instantiate_msg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmInstantiate {
            admin: None,
            label: format!("CALC-MF-FUND"),
            code_id: config.fund_code_id,
            funds: vec![],
            msg: to_binary(&FundInstantiateMsg {})?,
        }),
        AFTER_INSTANTIATE_FUND_FOR_MIGRATION_REPLY_ID,
    );

    Ok(Response::new()
        .add_attribute("method", "migrate_to_latest_code_id")
        .add_submessage(fund_instantiate_msg))
}
