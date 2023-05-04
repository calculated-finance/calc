use crate::helpers::validation::{assert_address_is_valid, assert_sender_is_admin};
use crate::state::pairs::save_pair;
use crate::{error::ContractError, types::pair::Pair};
use cosmwasm_std::{Addr, DepsMut};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{MessageInfo, Response};

pub fn create_pair_handler(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
    base_denom: String,
    quote_denom: String,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;
    assert_address_is_valid(deps.as_ref(), address.clone(), "pair address")?;

    let pair = Pair {
        address: address.clone(),
        base_denom: base_denom.clone(),
        quote_denom: quote_denom.clone(),
    };

    save_pair(deps.storage, &pair)?;

    Ok(Response::new()
        .add_attribute("method", "create_pair")
        .add_attribute("base_denom", base_denom)
        .add_attribute("quote_denom", quote_denom)
        .add_attribute("address", address))
}

#[cfg(test)]
mod create_pair_tests {
    use crate::{
        contract::execute,
        handlers::get_pairs::get_pairs_handler,
        msg::ExecuteMsg,
        state::pairs::find_pair,
        tests::{
            helpers::instantiate_contract,
            mocks::{calc_mock_dependencies, ADMIN, DENOM_UDEMO, DENOM_UKUJI},
        },
    };
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        Addr,
    };

    #[test]
    fn create_pair_with_valid_id_should_succeed() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let create_pair_execute_message = ExecuteMsg::CreatePair {
            address: Addr::unchecked("pair"),
            base_denom: DENOM_UDEMO.to_string(),
            quote_denom: DENOM_UKUJI.to_string(),
        };

        execute(deps.as_mut(), env, info, create_pair_execute_message).unwrap();

        let pair = &get_pairs_handler(deps.as_ref()).unwrap().pairs[0];

        assert_eq!(pair.base_denom, DENOM_UDEMO.to_string());
        assert_eq!(pair.quote_denom, DENOM_UKUJI.to_string());
        assert_eq!(pair.address, Addr::unchecked("pair"));
    }

    #[test]
    fn create_pair_that_already_exists_should_update_it() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let original_message = ExecuteMsg::CreatePair {
            base_denom: DENOM_UDEMO.to_string(),
            quote_denom: DENOM_UKUJI.to_string(),
            address: Addr::unchecked("pair-1"),
        };

        let message = ExecuteMsg::CreatePair {
            base_denom: DENOM_UDEMO.to_string(),
            quote_denom: DENOM_UKUJI.to_string(),
            address: Addr::unchecked("pair-2"),
        };

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            original_message.clone(),
        )
        .unwrap();

        execute(deps.as_mut(), env.clone(), info.clone(), original_message).unwrap();

        let original_pair = find_pair(
            deps.as_ref().storage,
            &[DENOM_UDEMO.to_string(), DENOM_UKUJI.to_string()],
        )
        .unwrap();

        execute(deps.as_mut(), env, info, message).unwrap();

        let pair = find_pair(
            deps.as_ref().storage,
            &[DENOM_UDEMO.to_string(), DENOM_UKUJI.to_string()],
        )
        .unwrap();

        assert_eq!(original_pair.address, Addr::unchecked("pair-1"));
        assert_eq!(pair.address, Addr::unchecked("pair-2"));
    }

    #[test]
    fn create_pair_with_unauthorised_sender_should_fail() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let info_with_unauthorised_sender = mock_info("not-admin", &vec![]);

        let create_pair_execute_message = ExecuteMsg::CreatePair {
            base_denom: String::from("base"),
            quote_denom: String::from("quote"),
            address: Addr::unchecked("pair"),
        };

        let result = execute(
            deps.as_mut(),
            env,
            info_with_unauthorised_sender,
            create_pair_execute_message,
        )
        .unwrap_err();

        assert_eq!(result.to_string(), "Unauthorized")
    }
}
