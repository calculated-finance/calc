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

use fund_core::msg::{
    ConfigResponse as FundConfigResponse, InstantiateMsg as FundInstantiateMsg,
    QueryMsg as FundQueryMsg,
};
use fund_router::msg::{
    ConfigResponse as RouterConfigResponse, FundResponse, QueryMsg as RouterQueryMsg,
};

pub fn migrate_to_latest_code_id(
    deps: DepsMut,
    info: MessageInfo,
    router: Addr,
) -> Result<Response, ContractError> {
    let get_router_config_response: RouterConfigResponse = deps
        .querier
        .query_wasm_smart(router.clone(), &RouterQueryMsg::GetConfig {})?;

    assert_sender_is_owner_or_admin(deps.storage, info.sender, &get_router_config_response)?;

    let get_fund_response = FundResponse {
        address: deps
            .querier
            .query_wasm_smart(router.clone(), &RouterQueryMsg::GetFund {})?,
    };

    let get_fund_config_response: FundConfigResponse = deps.querier.query_wasm_smart(
        get_fund_response.address.clone(),
        &FundQueryMsg::GetConfig {},
    )?;

    MIGRATION_CACHE.save(
        deps.storage,
        &MigrationCache {
            router_address: router.clone(),
            old_fund_address: get_fund_response.address,
            new_fund_address: None,
        },
    )?;

    let config = get_config(deps.storage)?;

    let fund_instantiate_msg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmInstantiate {
            admin: None,
            label: format!("CALC-MF-FUND"),
            code_id: config.fund_code_id,
            funds: vec![],
            msg: to_binary(&FundInstantiateMsg {
                router: router.clone(),
                swapper: get_fund_config_response.config.swapper,
                base_denom: get_fund_config_response.config.base_denom,
            })?,
        }),
        AFTER_INSTANTIATE_FUND_FOR_MIGRATION_REPLY_ID,
    );

    Ok(Response::new()
        .add_attribute("method", "migrate_to_latest_code_id")
        .add_submessage(fund_instantiate_msg))
}
