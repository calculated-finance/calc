use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr, Uint128,
};

use crate::{
    contract::{execute, instantiate},
    msg::{ExecuteMsg, InstantiateMsg},
};

pub const ADMIN: &str = "admin";
pub const USER: &str = "user";

#[test]
fn with_ukuji_creates_native_delegate_msg() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(USER, &[]);

    let instantiate_msg = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        allowed_z_callers: vec![Addr::unchecked(ADMIN), Addr::unchecked(USER)],
    };

    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    let zdelegate_msg = ExecuteMsg::ZDelegate {
        delegator_address: Addr::unchecked(USER),
        validator_address: Addr::unchecked("kujiraveloper"),
        denom: "ukuji".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let res = execute(deps.as_mut(), env, info, zdelegate_msg).unwrap();

    assert!(res
        .attributes
        .iter()
        .any(|attr| attr.key == "denom" && attr.value == "native"));
}

#[test]
fn with_other_denom_creates_alliance_delegate_msg() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(USER, &[]);

    let instantiate_msg = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        allowed_z_callers: vec![Addr::unchecked(ADMIN), Addr::unchecked(USER)],
    };

    instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    let zdelegate_msg = ExecuteMsg::ZDelegate {
        delegator_address: Addr::unchecked(USER),
        validator_address: Addr::unchecked("kujiraveloper"),
        denom: "demo".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let res = execute(deps.as_mut(), env, info, zdelegate_msg).unwrap();

    assert!(res
        .attributes
        .iter()
        .any(|attr| attr.key == "denom" && attr.value == "alliance"));
}
