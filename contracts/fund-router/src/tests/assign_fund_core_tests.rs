use cosmwasm_std::{
    from_binary,
    testing::{mock_dependencies, mock_env, mock_info},
    Addr,
};

use crate::{
    contract::{execute, query},
    msg::{ExecuteMsg, FundCoreResponse, QueryMsg},
};

use super::helpers::{instantiate_contract, FUND_CORE_ADDRESS, USER};

#[test]
fn with_valid_address_should_save_fund_core_address() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(USER, &vec![]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let assign_fund_core_msg = ExecuteMsg::AssignFundCore {
        fund_core_address: Addr::unchecked(FUND_CORE_ADDRESS),
    };

    execute(deps.as_mut(), env.clone(), info, assign_fund_core_msg).unwrap();

    let get_fund_core_query = QueryMsg::GetFundCore {};

    let binary = query(deps.as_ref(), env, get_fund_core_query).unwrap();

    let fund_core_response: FundCoreResponse = from_binary(&binary).unwrap();

    assert_eq!(
        fund_core_response.address,
        Addr::unchecked(FUND_CORE_ADDRESS)
    );
}

#[test]
fn with_invalid_address_should_fail() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(USER, &vec![]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let assign_fund_core_msg = ExecuteMsg::AssignFundCore {
        fund_core_address: Addr::unchecked(""),
    };

    let response = execute(deps.as_mut(), env.clone(), info, assign_fund_core_msg);

    assert!(response.is_err());
}

#[test]
fn multiple_fund_cores_only_returns_the_latest_core() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(USER, &vec![]);

    instantiate_contract(deps.as_mut(), env.clone(), info.clone());

    let assign_fund_core_msg = ExecuteMsg::AssignFundCore {
        fund_core_address: Addr::unchecked(FUND_CORE_ADDRESS),
    };

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        assign_fund_core_msg,
    )
    .unwrap();

    let assign_fund_core_msg = ExecuteMsg::AssignFundCore {
        fund_core_address: Addr::unchecked("fund_core_address_2"),
    };

    execute(deps.as_mut(), env.clone(), info, assign_fund_core_msg).unwrap();

    let get_fund_core_query = QueryMsg::GetFundCore {};

    let binary = query(deps.as_ref(), env, get_fund_core_query).unwrap();

    let fund_core_response: FundCoreResponse = from_binary(&binary).unwrap();

    assert_eq!(
        fund_core_response.address,
        Addr::unchecked("fund_core_address_2")
    );
}
