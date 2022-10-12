use base::helpers::message_helpers::{find_first_attribute_by_key, find_first_event_by_type};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StakingMsg, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Cache, Config, Delegation, CACHE, CONFIG, DELEGATIONS};

const CONTRACT_NAME: &str = "crates.io:compounder";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AFTER_DELEGATE_LOCAL: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let validated_address = deps.api.addr_validate(&msg.admin.to_string())?;

    let config = Config {
        admin: validated_address,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Delegate { validator_address } => delegate(deps, env, info, validator_address),
        ExecuteMsg::Undelegate {
            validator_address,
            amount,
        } => unimplemented!(),
        ExecuteMsg::Withdraw {
            validator_address,
            amount,
        } => unimplemented!(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_DELEGATE_LOCAL => after_delegate_local(deps, reply),
        id => Err(ContractError::CustomError {
            val: format!("unknown reply id: {}", id),
        }),
    }
}

fn delegate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    validator_address: Addr,
) -> Result<Response, ContractError> {
    let cache = Cache {
        funds: info.funds[0].clone(),
        validator_address,
    };
    CACHE.save(deps.storage, &cache)?;

    match info.funds[0].denom.contains("ibc") {
        true => delegate_ibc(validator_address, info.funds[0].clone()),
        false => delegate_local(validator_address, info.funds[0].clone()),
    }
}

fn delegate_local(validator_address: Addr, funds: Coin) -> Result<Response, ContractError> {
    let sub_msg = SubMsg::reply_always(
        StakingMsg::Delegate {
            validator: validator_address.to_string(),
            amount: funds.clone(),
        },
        AFTER_DELEGATE_LOCAL,
    );
    Ok(Response::new().add_submessage(sub_msg))
}

fn delegate_ibc(validator_address: Addr, funds: Coin) -> Result<Response, ContractError> {
    unimplemented!()
}

fn after_delegate_local(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            let delegate_response = reply.result.into_result().unwrap();
            let delegate_event =
                find_first_event_by_type(&delegate_response.events, String::from("delegate"))
                    .unwrap();

            let new_shares =
                find_first_attribute_by_key(&delegate_event.attributes, String::from("new_shares"))
                    .unwrap()
                    .value
                    .parse::<Decimal>()
                    .unwrap();

            let updated_delegation = DELEGATIONS.update(
                deps.storage,
                cache.validator_address.clone(),
                |existing_delegation: Option<Delegation>| -> StdResult<Delegation> {
                    match existing_delegation {
                        Some(mut delegation) => {
                            delegation.amount += cache.funds.amount.clone();
                            delegation.shares =
                                delegation.shares.checked_add(new_shares.clone()).unwrap();
                            Ok(delegation)
                        }
                        None => {
                            let delegation = Delegation {
                                amount: cache.amount,
                                shares: new_shares.clone(),
                                validator_address: cache.validator_address,
                            };
                            Ok(delegation)
                        }
                    }
                },
            )?;

            Ok(Response::new()
                .add_attribute("method", "after_deposit")
                .add_attribute("shares", updated_delegation.shares.to_string()))
        }
        cosmwasm_std::SubMsgResult::Err(e) => Ok(Response::new()
            .add_attribute("method", "after_deposit")
            .add_attribute("error", e)),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
