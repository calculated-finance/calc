use base::helpers::message_helpers::{find_first_attribute_by_key, find_first_event_by_type};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, Delegation, Deps, DepsMut, DistributionMsg, Env,
    MessageInfo, Reply, Response, StakingMsg, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Cache, CompoundingDelegation, Config, CACHE, CONFIG, DELEGATIONS};

const CONTRACT_NAME: &str = "crates.io:compounder";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AFTER_DELEGATE_LOCAL: u64 = 1;
const AFTER_CLAIM_REWARD_LOCAL: u64 = 2;
const AFTER_UNDELEGATE_LOCAL: u64 = 3;

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
            denom,
            amount,
        } => undelegate(deps, validator_address, denom, amount),
        ExecuteMsg::Withdraw { to_address } => withdraw(deps, env, to_address),
        ExecuteMsg::Compound {} => compound(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_DELEGATE_LOCAL => after_delegate_local(deps, reply),
        AFTER_CLAIM_REWARD_LOCAL => after_claim_reward_local(deps, reply),
        AFTER_UNDELEGATE_LOCAL => after_undelegate_local(deps, reply),
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
    Ok(Response::new().add_submessage(delegate_msg))
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
            let updated_delegation = DELEGATIONS.update(
                deps.storage,
                cache.validator_address.clone(),
                |existing_delegation: Option<CompoundingDelegation>| -> StdResult<CompoundingDelegation> {
                    match existing_delegation {
                        Some(mut delegation) => {
                            delegation.balance.amount += cache.funds.amount.clone();
                            Ok(delegation)
                        }
                        None => {
                            let delegation = CompoundingDelegation {
                                balance: cache.funds,
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
                .add_attribute("balance", updated_delegation.balance.to_string()))
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
            let claim_reward_sub_msg =
                match delegation.validator_address.to_string().contains("kujira") {
                    true => claim_reward_local(delegation.validator_address),
                    false => claim_reward_ibc(delegation.validator_address),
                };

            claim_reward_sub_msg
        })
        .collect();

    Ok(Response::new()
        .add_attribute("method", "compound")
        .add_submessages(claim_reward_sub_msgs))
}

fn claim_reward_local(validator_address: Addr) -> SubMsg {
    let msg = DistributionMsg::WithdrawDelegatorReward {
        validator: validator_address.to_string(),
    };
    SubMsg::reply_always(msg, AFTER_CLAIM_REWARD_LOCAL)
}

fn claim_reward_ibc(_validator_address: Addr) -> SubMsg {
    unimplemented!()
}

fn after_claim_reward_local(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            let claim_reward_response = reply.result.into_result().unwrap();
            let withdraw_rewards_event = find_first_event_by_type(
                &claim_reward_response.events,
                String::from("withdraw_rewards"),
            )
            .unwrap();

            let validator_address = find_first_attribute_by_key(
                &withdraw_rewards_event.attributes,
                String::from("validator"),
            )
            .unwrap()
            .value
            .clone();

            let validator_address = Addr::unchecked(validator_address);

            let reward_amount_with_denom = find_first_attribute_by_key(
                &withdraw_rewards_event.attributes,
                String::from("amount"),
            )
            .unwrap()
            .value
            .clone();

            let reward_amount: String = reward_amount_with_denom
                .chars()
                .filter(|char| char.is_numeric())
                .collect();

            let funds = Coin {
                denom: String::from("ukuji"),
                amount: reward_amount.parse::<Uint128>().unwrap(),
            };

            let delegate_msg = delegate_local(validator_address.clone(), funds.clone());

            // save amounts to contract to be updated after delegation
            let cache = Cache {
                funds: funds.clone(),
                validator_address: validator_address.clone(),
            };

            CACHE.save(deps.storage, &cache)?;

            Ok(Response::new()
                .add_attribute("method", "after_claim_reward_local")
                .add_submessage(delegate_msg))
        }
        cosmwasm_std::SubMsgResult::Err(e) => Ok(Response::new()
            .add_attribute("method", "after_claim_reward_local")
            .add_attribute("error", e)),
    }
}

fn undelegate(
    deps: DepsMut,
    validator_address: Addr,
    denom: String,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let amount = match amount {
        Some(amount) => Coin {
            denom: denom.clone(),
            amount,
        },
        None => {
            let delegation =
                DELEGATIONS.load(deps.storage, Addr::unchecked(validator_address.clone()))?;
            Coin {
                denom: denom.clone(),
                amount: delegation.balance.amount,
            }
        }
    };

    let cache = Cache {
        funds: amount.clone(),
        validator_address: validator_address.clone(),
    };
    CACHE.save(deps.storage, &cache)?;

    let undelegate_msg = match denom.contains("ibc") {
        true => undelegate_ibc(validator_address, amount),
        false => undelegate_local(validator_address, amount),
    };

    Ok(Response::new()
        .add_attribute("method", "undelegate")
        .add_submessage(undelegate_msg))
}

fn undelegate_local(validator_address: Addr, amount: Coin) -> SubMsg {
    SubMsg::reply_always(
        StakingMsg::Undelegate {
            validator: validator_address.to_string(),
            amount: amount.clone(),
        },
        AFTER_UNDELEGATE_LOCAL,
    )
}

fn undelegate_ibc(_validator_address: Addr, _amount: Coin) -> SubMsg {
    unimplemented!()
}

fn after_undelegate_local(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            let undelegate_response = reply.result.into_result().unwrap();
            let undelegate_event =
                find_first_event_by_type(&undelegate_response.events, String::from("unbond"))
                    .unwrap();

            let completion_time = find_first_attribute_by_key(
                &undelegate_event.attributes,
                String::from("completion_time"),
            )
            .unwrap()
            .value
            .clone();

            let updated_delegation = DELEGATIONS.update(
                deps.storage,
                cache.validator_address.clone(),
                |existing_delegation: Option<CompoundingDelegation>| -> StdResult<CompoundingDelegation> {
                    match existing_delegation {
                        Some(mut delegation) => {
                            delegation.balance.amount -= cache.funds.amount.clone();
                            Ok(delegation)
                        }
                        None => {
                            let delegation = CompoundingDelegation {
                                balance: cache.funds,
                                validator_address: cache.validator_address,
                            };
                            Ok(delegation)
                        }
                    }
                },
            )?;

            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_undelegate_local")
                .add_attribute("balance", updated_delegation.balance.to_string())
                .add_attribute("completion_time", completion_time))
        }
        cosmwasm_std::SubMsgResult::Err(e) => {
            CACHE.remove(deps.storage);
            Ok(Response::new()
                .add_attribute("method", "after_undelegate_local")
                .add_attribute("error", e))
        }
    }
}

fn withdraw(deps: DepsMut, env: Env, to_address: Addr) -> Result<Response, ContractError> {
    let balances = deps
        .querier
        .query_all_balances(env.contract.address)
        .unwrap();

    Ok(Response::new()
        .add_attribute("method", "withdraw")
        .add_message(BankMsg::Send {
            to_address: to_address.to_string(),
            amount: balances,
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalances {} => to_binary(&get_balances(deps, env)?),
        QueryMsg::GetUnbondingDelegations {} => unimplemented!(),
        QueryMsg::GetDelegations {} => to_binary(&get_delegations(deps, env)?),
    }
}

fn get_balances(deps: Deps, env: Env) -> StdResult<Vec<Coin>> {
    let balances = deps
        .querier
        .query_all_balances(env.contract.address)
        .unwrap();

    Ok(balances)
}

fn get_delegations(deps: Deps, env: Env) -> StdResult<Vec<Delegation>> {
    let delegations = deps
        .querier
        .query_all_delegations(env.contract.address)
        .unwrap();

    Ok(delegations)
}

#[cfg(test)]
mod tests {}
