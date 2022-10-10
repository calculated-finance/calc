use base::helpers::message_helpers::{find_first_attribute_by_key, find_first_event_by_type};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint64,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Cache, CACHE, CONTRACTS_BY_ADDRESS, CONTRACT_CODE_ID};
use crate::validation_helpers::{assert_denom_is_kuji, assert_exactly_one_asset};

use compound_child::msg::{ExecuteMsg as ExecuteMsgChild, InstantiateMsg as InstantiateMsgChild};

const CONTRACT_NAME: &str = "crates.io:compounder";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AFTER_CREATE_CONTRACT: u64 = 1;
const AFTER_DEPOSIT: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Stake {
            delegator_address,
            validator_address,
        } => stake(deps, env, info, delegator_address, validator_address),
        ExecuteMsg::SetCodeId { code_id } => set_code_id(deps, code_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_CREATE_CONTRACT => after_create_contract(deps, reply),
        AFTER_DEPOSIT => after_deposit(deps, reply),
        id => Err(ContractError::CustomError {
            val: format!("unknown reply id: {}", id),
        }),
    }
}

fn set_code_id(deps: DepsMut, code_id: Uint64) -> Result<Response, ContractError> {
    CONTRACT_CODE_ID.save(deps.storage, &code_id.u64())?;
    Ok(Response::new().add_attribute("method", "set_code_id"))
}

fn stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    delegator_address: Addr,
    validator_address: Addr,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;
    assert_denom_is_kuji(info.funds.clone())?;

    let validated_address = deps.api.addr_validate(&delegator_address.as_str())?;
    let staking_contract = CONTRACTS_BY_ADDRESS.load(deps.storage, validated_address);

    let msg = match staking_contract {
        Ok(contract_address) => SubMsg::reply_always(
            deposit(
                contract_address,
                validator_address.clone(),
                info.funds[0].clone(),
            )?,
            AFTER_DEPOSIT,
        ),
        Err(_) => {
            let code_id = CONTRACT_CODE_ID.load(deps.storage)?;
            SubMsg::reply_always(
                create_contract(code_id, env.contract.address, delegator_address.clone())?,
                AFTER_CREATE_CONTRACT,
            )
        }
    };

    let cache = Cache {
        owner: delegator_address,
        funds: info.funds[0].clone(),
        validator_address,
    };

    CACHE.save(deps.storage, &cache)?;

    Ok(Response::new().add_submessage(msg))
}

fn deposit(contract_address: Addr, validator_address: Addr, funds: Coin) -> StdResult<CosmosMsg> {
    let deposit = ExecuteMsgChild::Deposit {
        validator_address: validator_address.clone(),
    };

    Ok(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: contract_address.to_string(),
        msg: to_binary(&deposit)?,
        funds: vec![funds.clone()],
    }))
}

fn after_deposit(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_deposit")
                .add_attribute("status", "success"))
        }
        cosmwasm_std::SubMsgResult::Err(e) => {
            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_deposit")
                .add_attribute("status", "fail")
                .add_attribute("error", e))
        }
    }
}

fn create_contract(code_id: u64, admin: Addr, owner: Addr) -> StdResult<CosmosMsg> {
    let instantiate = InstantiateMsgChild {
        admin: admin.clone(),
    };

    Ok(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
        admin: Some(admin.to_string()),
        code_id,
        msg: to_binary(&instantiate)?,
        funds: vec![],
        label: format!("calc-compounder - {}", owner),
    }))
}

fn after_create_contract(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            let instantiate_response = reply.result.into_result().unwrap();

            let instantiate_event =
                find_first_event_by_type(&instantiate_response.events, String::from("instantiate"))
                    .unwrap();

            let contract_address = find_first_attribute_by_key(
                &instantiate_event.attributes,
                String::from("_contract_address"),
            )
            .unwrap()
            .value
            .clone();

            let validated_address = deps.api.addr_validate(&contract_address)?;

            let cache = CACHE.load(deps.storage)?;

            CONTRACTS_BY_ADDRESS.save(deps.storage, cache.owner, &validated_address.clone())?;

            let msg = SubMsg::reply_always(
                deposit(
                    validated_address,
                    cache.validator_address.clone(),
                    cache.funds.clone(),
                )?,
                AFTER_DEPOSIT,
            );

            Ok(Response::new()
                .add_attribute("method", "after_create_contract")
                .add_attribute("status", "success")
                .add_submessage(msg))
        }
        cosmwasm_std::SubMsgResult::Err(e) => {
            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_create_contract")
                .add_attribute("status", "fail")
                .add_attribute("error", e))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
