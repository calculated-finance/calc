use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr,
};

use crate::contract::instantiate;
use crate::msg::InstantiateMsg;

pub const ADMIN: &str = "admin";

#[test]
fn with_valid_admin_should_succeed() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(ADMIN, &vec![]);

    let msg = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
    };

    let res = instantiate(deps.as_mut(), env, info, msg);

    assert!(res.is_ok())
}
