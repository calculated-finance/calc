use base::ContractError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::handlers::get_config::get_config_handler;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{update_config, Config};
use crate::validation::assert_sender_is_router;

pub type ContractResult<T> = Result<T, ContractError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    update_config(
        deps.storage,
        &Config {
            router: msg.router.clone(),
            swapper: msg.swapper.clone(),
            base_denom: msg.base_denom.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("router", msg.router.to_string())
        .add_attribute("swap", msg.swapper.to_string())
        .add_attribute("base_asset", msg.base_denom.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> ContractResult<Response> {
    match _msg {
        ExecuteMsg::Migrate { new_fund_address } => {
            assert_sender_is_router(deps.storage, info.sender.clone())?;

            let balance = deps.querier.query_all_balances(new_fund_address)?;
            Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: balance,
                })
                .add_attribute("method", "migrate"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&get_config_handler(deps)?),
    }
}
