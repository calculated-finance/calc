use base::helpers::message_helpers::{find_first_attribute_by_key, find_first_event_by_type};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response, StakingMsg,
    StdResult, SubMsg, DistributionMsg, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Cache, Config, Delegation, CACHE, CONFIG, DELEGATIONS};

const CONTRACT_NAME: &str = "crates.io:compounder";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AFTER_DELEGATE_LOCAL: u64 = 1;
const AFTER_CLAIM_REWARD_LOCAL: u64 = 2;

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
            validator_address: _,
            amount: _,
        } => unimplemented!(),
        ExecuteMsg::Withdraw {
            validator_address: _,
            amount: _,
        } => unimplemented!(),
        ExecuteMsg::Compound {} => compound(deps, env, info)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_DELEGATE_LOCAL => after_delegate_local(deps, reply),
        AFTER_CLAIM_REWARD_LOCAL => after_claim_reward_local(deps, reply),
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
        validator_address: validator_address.clone(),
    };
    CACHE.save(deps.storage, &cache)?;

    let delegate_msg = match info.funds[0].denom.contains("ibc") {
        true => delegate_ibc(validator_address, info.funds[0].clone()),
        false => delegate_local(validator_address, info.funds[0].clone()),
    };
    Ok(
        Response::new()
        .add_submessage(delegate_msg)
    )
}

fn delegate_local(validator_address: Addr, funds: Coin) -> SubMsg {
    SubMsg::reply_always(
        StakingMsg::Delegate {
            validator: validator_address.to_string(),
            amount: funds.clone(),
        },
        AFTER_DELEGATE_LOCAL,
    )
}

fn delegate_ibc(_validator_address: Addr, _funds: Coin) -> SubMsg {
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
                            delegation.balance.amount += cache.funds.amount.clone();
                            delegation.shares =
                                delegation.shares.checked_add(new_shares.clone()).unwrap();
                            Ok(delegation)
                        }
                        None => {
                            let delegation = Delegation {
                                balance: cache.funds,
                                shares: new_shares.clone(),
                                validator_address: cache.validator_address,
                            };
                            Ok(delegation)
                        }
                    }
                },
            )?;

            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_delegate_local")
                .add_attribute("balance", updated_delegation.balance.to_string())
                .add_attribute("shares", updated_delegation.shares.to_string()))
        }
        cosmwasm_std::SubMsgResult::Err(e) => { 
            CACHE.remove(deps.storage);
            Ok(Response::new()
            .add_attribute("method", "after_delegate_local")
            .add_attribute("error", e)) 
        }
    }
}

fn compound(deps: DepsMut, _env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let claim_reward_sub_msgs: Vec<SubMsg> = DELEGATIONS
    .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
    .map(|d| {
        let delegation = d.unwrap().1;
        let claim_reward_sub_msg = match delegation.validator_address.to_string().contains("kujira") {
            true => claim_reward_local(delegation.validator_address),
            false => claim_reward_ibc(delegation.validator_address)
        };

        claim_reward_sub_msg
    })
    .collect();
    
    Ok(
        Response::new()
        .add_attribute("method", "compound")
        .add_submessages(claim_reward_sub_msgs)
    )
}

fn claim_reward_local(validator_address: Addr) -> SubMsg {
    let msg = DistributionMsg::WithdrawDelegatorReward { validator: validator_address.to_string() };
    SubMsg::reply_always(msg, AFTER_CLAIM_REWARD_LOCAL)
}

fn claim_reward_ibc(_validator_address: Addr) -> SubMsg {
    unimplemented!()
}

fn after_claim_reward_local(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            let claim_reward_response = reply.result.into_result().unwrap();
            let withdraw_rewards_event = find_first_event_by_type(&claim_reward_response.events, String::from("withdraw_rewards"))
                .unwrap();
            
            let validator_address = find_first_attribute_by_key(&withdraw_rewards_event.attributes, String::from("validator"))
                .unwrap()
                .value
                .clone();

            let validator_address = Addr::unchecked(validator_address);

            let reward_amount_with_denom = find_first_attribute_by_key(&withdraw_rewards_event.attributes, String::from("amount"))
                .unwrap()
                .value
                .clone();
            

            let reward_amount: String = reward_amount_with_denom
                .chars()
                .filter(|char| char.is_numeric())
                .collect();

            let funds = Coin { denom: String::from("ukuji"), amount: reward_amount.parse::<Uint128>().unwrap() };
            
            let delegate_msg = delegate_local(validator_address.clone(), funds.clone());

            // save amounts to contract to be updated after delegation
            let cache = Cache {
                funds: funds.clone(),
                validator_address: validator_address.clone(),
            };

            CACHE.save(deps.storage, &cache)?;

            Ok(
                Response::new()
                .add_attribute("method", "after_claim_reward_local")
                .add_submessage(delegate_msg)
            )
        },
        cosmwasm_std::SubMsgResult::Err(e) => {
            Ok(
                Response::new()
                .add_attribute("method", "after_claim_reward_local")
                .add_attribute("error", e)
            )
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
