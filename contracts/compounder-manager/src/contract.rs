use base::helpers::message_helpers::{find_first_attribute_by_key, find_first_event_by_type};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint128, Uint64, Delegation,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CompoundersResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Cache, CACHE, COMPOUNDER_CONTRACTS_BY_ADDRESS, COMPOUNDER_CONTRACT_CODE_ID};
use crate::validation_helpers::assert_exactly_one_asset;

use compounder::msg::{
    ExecuteMsg as CompounderExecuteMsg, InstantiateMsg as CompounderInstantiateMsg,
    QueryMsg as CompounderQueryMsg,
};

const CONTRACT_NAME: &str = "crates.io:compounder-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AFTER_CREATE_COMPOUNDER_CONTRACT: u64 = 1;
const AFTER_DELEGATE_TO_COMPOUNDER: u64 = 2;
const AFTER_UNDELEGATE_FROM_COMPOUNDER: u64 = 3;

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
        ExecuteMsg::Delegate {
            delegator_address,
            validator_address,
        } => delegate(deps, env, info, delegator_address, validator_address),
        ExecuteMsg::Undelegate {
            delegator_address,
            validator_address,
            denom,
            amount,
        } => undelegate(deps, delegator_address, validator_address, denom, amount),
        ExecuteMsg::SetCompounderCodeId { code_id } => set_compounder_code_id(deps, code_id),
        ExecuteMsg::Withdraw {
            delegator_address,
            to_address,
        } => withdraw(deps, delegator_address, to_address),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_CREATE_COMPOUNDER_CONTRACT => after_create_compounder_contract(deps, reply),
        AFTER_DELEGATE_TO_COMPOUNDER => after_delegate_to_compounder(deps, reply),
        AFTER_UNDELEGATE_FROM_COMPOUNDER => after_undelegate_from_compounder(deps, reply),
        id => Err(ContractError::CustomError {
            val: format!("unknown reply id: {}", id),
        }),
    }
}

fn set_compounder_code_id(deps: DepsMut, code_id: Uint64) -> Result<Response, ContractError> {
    COMPOUNDER_CONTRACT_CODE_ID.save(deps.storage, &code_id.u64())?;
    Ok(Response::new().add_attribute("method", "set_code_id"))
}

fn delegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    delegator_address: Addr,
    validator_address: Addr,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;

    let validated_address = deps.api.addr_validate(&delegator_address.as_str())?;
    let compounder_contract = COMPOUNDER_CONTRACTS_BY_ADDRESS.load(deps.storage, validated_address);

    let msg = match compounder_contract {
        Ok(contract_address) => delegate_to_compounder(
            contract_address,
            validator_address.clone(),
            info.funds[0].clone(),
        ),
        Err(_) => {
            let code_id = COMPOUNDER_CONTRACT_CODE_ID.load(deps.storage)?;
            create_compounder_contract(code_id, env.contract.address, delegator_address.clone())
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

fn delegate_to_compounder(contract_address: Addr, validator_address: Addr, funds: Coin) -> SubMsg {
    let delegate_msg = CompounderExecuteMsg::Delegate {
        validator_address: validator_address.clone(),
    };

    SubMsg::reply_always(
        CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: contract_address.to_string(),
            msg: to_binary(&delegate_msg).unwrap(),
            funds: vec![funds.clone()],
        }),
        AFTER_DELEGATE_TO_COMPOUNDER,
    )
}

fn after_delegate_to_compounder(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            CACHE.remove(deps.storage);

            Ok(Response::new().add_attribute("method", "after_delegate_to_compounder"))
        }
        cosmwasm_std::SubMsgResult::Err(e) => {
            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_delegate_to_compounder")
                .add_attribute("error", e))
        }
    }
}

fn create_compounder_contract(code_id: u64, admin: Addr, owner: Addr) -> SubMsg {
    let instantiate = CompounderInstantiateMsg {
        admin: admin.clone(),
    };

    SubMsg::reply_always(
        CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
            admin: Some(admin.to_string()),
            code_id,
            msg: to_binary(&instantiate).unwrap(),
            funds: vec![],
            label: format!("calc-compounder - {}", owner),
        }),
        AFTER_CREATE_COMPOUNDER_CONTRACT,
    )
}

fn after_create_compounder_contract(
    deps: DepsMut,
    reply: Reply,
) -> Result<Response, ContractError> {
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

            COMPOUNDER_CONTRACTS_BY_ADDRESS.save(
                deps.storage,
                cache.owner,
                &validated_address.clone(),
            )?;

            let msg = delegate_to_compounder(
                validated_address,
                cache.validator_address.clone(),
                cache.funds.clone(),
            );

            Ok(Response::new()
                .add_attribute("method", "after_create_compounder_contract")
                .add_submessage(msg))
        }
        cosmwasm_std::SubMsgResult::Err(e) => {
            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_create_compounder_contract")
                .add_attribute("error", e))
        }
    }
}

fn undelegate(
    deps: DepsMut,
    delegator_address: Addr,
    validator_address: Addr,
    denom: String,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let compounder_contract = COMPOUNDER_CONTRACTS_BY_ADDRESS
        .load(deps.storage, Addr::unchecked(delegator_address.clone()))?;

    let undelegate_msg =
        undelegate_from_compounder(compounder_contract, validator_address, denom, amount);

    Ok(Response::new()
        .add_attribute("method", "undelegate")
        .add_submessage(undelegate_msg))
}

fn undelegate_from_compounder(
    contract_address: Addr,
    validator_address: Addr,
    denom: String,
    amount: Option<Uint128>,
) -> SubMsg {
    let undelegate_msg = CompounderExecuteMsg::Undelegate {
        validator_address,
        denom,
        amount,
    };
    SubMsg::reply_always(
        CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: contract_address.to_string(),
            msg: to_binary(&undelegate_msg).unwrap(),
            funds: vec![],
        }),
        AFTER_UNDELEGATE_FROM_COMPOUNDER,
    )
}

fn after_undelegate_from_compounder(
    _deps: DepsMut,
    reply: Reply,
) -> Result<Response, ContractError> {
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            Ok(Response::new().add_attribute("method", "after_undelegate_from_compounder"))
        }
        cosmwasm_std::SubMsgResult::Err(e) => Ok(Response::new()
            .add_attribute("method", "after_undelegate_from_compounder")
            .add_attribute("error", e)),
    }
}

fn withdraw(
    deps: DepsMut,
    delegator_address: Addr,
    to_address: Addr,
) -> Result<Response, ContractError> {
    let compounder_contract = COMPOUNDER_CONTRACTS_BY_ADDRESS
        .load(deps.storage, Addr::unchecked(delegator_address.clone()))?;

    let withdraw_msg = withdraw_from_compounder(compounder_contract, to_address);

    Ok(Response::new()
        .add_attribute("method", "withdraw")
        .add_message(withdraw_msg))
}

fn withdraw_from_compounder(contract_address: Addr, to_address: Addr) -> CosmosMsg {
    let withdraw_msg = CompounderExecuteMsg::Withdraw { to_address };

    CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: contract_address.to_string(),
        msg: to_binary(&withdraw_msg).unwrap(),
        funds: vec![],
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCompounders {} => to_binary(&get_compounders(deps)?),
        QueryMsg::GetBalances { delegator_address } => {
            to_binary(&get_balances(deps, delegator_address)?)
        },
        QueryMsg::GetDelegations { delegator_address } => to_binary(&get_delegations(deps, delegator_address)?),
        QueryMsg::GetUnbondingDelegations {} => unimplemented!(),
    }
}

fn get_compounders(deps: Deps) -> StdResult<CompoundersResponse> {
    let compounders: Vec<Addr> = COMPOUNDER_CONTRACTS_BY_ADDRESS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|c| c.unwrap().1)
        .collect();

    Ok(CompoundersResponse { compounders })
}

fn get_delegations(deps: Deps, delegator_address: Addr) -> StdResult<Vec<Delegation>> {
    let compounder_contract = COMPOUNDER_CONTRACTS_BY_ADDRESS
        .load(deps.storage, Addr::unchecked(delegator_address.clone()))?;

    let get_delegations_msg = CompounderQueryMsg::GetDelegations {};

    let delegations: Vec<Delegation> = deps
        .querier
        .query_wasm_smart(compounder_contract, &get_delegations_msg)
        .unwrap();

    Ok(delegations)
}


fn get_balances(deps: Deps, delegator_address: Addr) -> StdResult<Vec<Coin>> {
    let compounder_contract = COMPOUNDER_CONTRACTS_BY_ADDRESS
        .load(deps.storage, Addr::unchecked(delegator_address.clone()))?;

    let get_balances_msg = CompounderQueryMsg::GetBalances {};

    let balances: Vec<Coin> = deps
        .querier
        .query_wasm_smart(compounder_contract, &get_balances_msg)
        .unwrap();

    Ok(balances)
}
