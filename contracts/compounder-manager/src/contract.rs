use base::helpers::message_helpers::{find_first_attribute_by_key, find_first_event_by_type};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint64,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CompoundersResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Cache, CACHE, COMPOUNDER_CONTRACTS_BY_ADDRESS, COMPOUNDER_CONTRACT_CODE_ID};
use crate::validation_helpers::{assert_denom_is_kuji, assert_exactly_one_asset};

use compounder::msg::{
    ExecuteMsg as CompounderExecuteMsg, InstantiateMsg as CompounderInstantiateMsg,
};

const CONTRACT_NAME: &str = "crates.io:compounder-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AFTER_CREATE_COMPOUNDER_CONTRACT: u64 = 1;
const AFTER_DEPOSIT_TO_COMPOUNDER: u64 = 2;

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
            delegator_address: _,
            validator_address: _,
            amount: _,
        } => unimplemented!(),
        ExecuteMsg::SetCompounderCodeId { code_id } => set_compounder_code_id(deps, code_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        AFTER_CREATE_COMPOUNDER_CONTRACT => after_create_compounder_contract(deps, reply),
        AFTER_DEPOSIT_TO_COMPOUNDER => after_deposit_to_compounder(deps, reply),
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
    assert_denom_is_kuji(info.funds.clone())?;

    let validated_address = deps.api.addr_validate(&delegator_address.as_str())?;
    let compounder_contract = COMPOUNDER_CONTRACTS_BY_ADDRESS.load(deps.storage, validated_address);

    let msg = match compounder_contract {
        Ok(contract_address) => SubMsg::reply_always(
            deposit_to_compounder(
                contract_address,
                validator_address.clone(),
                info.funds[0].clone(),
            )?,
            AFTER_DEPOSIT_TO_COMPOUNDER,
        ),
        Err(_) => {
            let code_id = COMPOUNDER_CONTRACT_CODE_ID.load(deps.storage)?;
            SubMsg::reply_always(
                create_compounder_contract(
                    code_id,
                    env.contract.address,
                    delegator_address.clone(),
                )?,
                AFTER_CREATE_COMPOUNDER_CONTRACT,
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

fn deposit_to_compounder(
    contract_address: Addr,
    validator_address: Addr,
    funds: Coin,
) -> StdResult<CosmosMsg> {
    let deposit = CompounderExecuteMsg::Delegate {
        validator_address: validator_address.clone(),
    };

    Ok(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: contract_address.to_string(),
        msg: to_binary(&deposit)?,
        funds: vec![funds.clone()],
    }))
}

fn after_deposit_to_compounder(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            CACHE.remove(deps.storage);

            Ok(Response::new().add_attribute("method", "after_deposit_to_compounder"))
        }
        cosmwasm_std::SubMsgResult::Err(e) => {
            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute("method", "after_deposit_to_compounder")
                .add_attribute("error", e))
        }
    }
}

fn create_compounder_contract(code_id: u64, admin: Addr, owner: Addr) -> StdResult<CosmosMsg> {
    let instantiate = CompounderInstantiateMsg {
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

            let msg = SubMsg::reply_always(
                deposit_to_compounder(
                    validated_address,
                    cache.validator_address.clone(),
                    cache.funds.clone(),
                )?,
                AFTER_DEPOSIT_TO_COMPOUNDER,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCompounders {} => to_binary(&get_compounders(deps)?),
        QueryMsg::GetBalances {} => unimplemented!(),
    }
}

fn get_compounders(deps: Deps) -> StdResult<CompoundersResponse> {
    let compounders: Vec<Addr> = COMPOUNDER_CONTRACTS_BY_ADDRESS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|c| c.unwrap().1)
        .collect();

    Ok(CompoundersResponse { compounders })
}
