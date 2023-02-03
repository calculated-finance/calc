use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    SubMsg,
};
use kujira::{denom::Denom, msg::DenomMsg};

use crate::{contract::instantiate, msg::InstantiateMsg};

pub const USER: &str = "user";

#[test]
fn instantiate_contract_creates_new_denom() {
    let mut deps = mock_dependencies();
    let mock_env = mock_env();
    let info = mock_info(USER, &[]);

    let instantiate_msg = InstantiateMsg {
        token_name: "test_token".to_string(),
    };

    let response = instantiate(deps.as_mut(), mock_env.clone(), info, instantiate_msg).unwrap();

    assert!(response.messages.contains(&SubMsg::new(DenomMsg::Create {
        subdenom: Denom::from("test_token"),
    })));
}
