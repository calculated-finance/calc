#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::handlers::assign_fund_to_router::assign_fund_to_router;
//use crate::handlers::assign_new_fund_to_router::assign_new_fund_to_router;
use crate::handlers::create_router::create_router;
use crate::handlers::get_config::get_config_handler;
use crate::handlers::get_routers_by_address::get_routers_by_address_handler;
use crate::handlers::migrate_fund::migrate_fund;
use crate::handlers::save_router::save_router_handler;
//use crate::handlers::transfer_all_fund_assets::transfer_all_fund_assets;
use crate::handlers::update_config::update_config_handler;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::config::{update_config, Config};

use base::ContractError;

const CONTRACT_NAME: &str = "crates.io:fund-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(&msg.admin.to_string())?;

    update_config(
        deps.storage,
        Config {
            admin: msg.admin,
            router_code_id: msg.router_code_id,
            fund_code_id: msg.fund_code_id,
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateRouter { token_name } => create_router(deps, env, info, token_name),
        ExecuteMsg::UpdateConfig {
            admin,
            router_code_id,
            fund_code_id,
        } => update_config_handler(deps, info, admin, router_code_id, fund_code_id),
        ExecuteMsg::MigrateToLatestCodeId { router } => migrate_fund(deps, info, router),
    }
}

pub const AFTER_INSTANTIATE_ROUTER_REPLY_ID: u64 = 1;
pub const AFTER_INSTANTIATE_FUND_REPLY_ID: u64 = 2;
pub const AFTER_INSTANTIATE_FUND_FOR_MIGRATION_REPLY_ID: u64 = 3;
pub const AFTER_TRANSFER_ALL_FUND_ASSETS_REPLY_ID: u64 = 4;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_INSTANTIATE_ROUTER_REPLY_ID => save_router_handler(deps, reply),
        AFTER_INSTANTIATE_FUND_REPLY_ID => assign_fund_to_router(deps, reply),
        AFTER_INSTANTIATE_FUND_FOR_MIGRATION_REPLY_ID => assign_fund_to_router(deps, reply),
        //AFTER_TRANSFER_ALL_FUND_ASSETS_REPLY_ID => assign_new_fund_to_router(deps, reply),
        id => Err(ContractError::CustomError {
            val: format!("unknown reply id: {}", id),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&get_config_handler(deps)?),
        QueryMsg::GetRouters { owner } => to_binary(&get_routers_by_address_handler(deps, owner)?),
    }
}
