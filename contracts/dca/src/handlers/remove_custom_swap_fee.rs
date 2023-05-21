use crate::helpers::validation::assert_sender_is_admin;
use crate::state::custom_fees::remove_custom_fee;
use crate::{error::ContractError, state::custom_fees::get_custom_fee};
use cosmwasm_std::DepsMut;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{MessageInfo, Response};

pub fn remove_custom_swap_fee_handler(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;

    let fee = get_custom_fee(deps.storage, denom.clone())?;

    if fee.is_none() {
        return Err(ContractError::CustomError {
            val: format!("Custom fee for {} does not exist", denom),
        });
    }

    remove_custom_fee(deps.storage, denom.clone());

    Ok(Response::new()
        .add_attribute("method", "remove_custom_swap_fee")
        .add_attribute("denom", denom))
}

#[cfg(test)]
mod remove_custom_swap_fee_tests {
    use super::*;
    use crate::{
        handlers::{
            create_custom_swap_fee::create_custom_swap_fee_handler,
            get_custom_swap_fees::get_custom_swap_fees_handler,
        },
        state::pairs::save_pair,
        tests::{
            helpers::instantiate_contract,
            mocks::{ADMIN, DENOM_UUSK},
        },
        types::pair::Pair,
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Decimal,
    };

    #[test]
    fn without_custom_fee_fails() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let denom = DENOM_UUSK.to_string();

        let err = remove_custom_swap_fee_handler(deps.as_mut(), info, denom).unwrap_err();

        assert_eq!(
            err.to_string(),
            format!("Error: Custom fee for {} does not exist", DENOM_UUSK)
        );
    }

    #[test]
    fn with_custom_fee_succeeds() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &vec![]);

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let denom = DENOM_UUSK.to_string();

        save_pair(deps.as_mut().storage, &Pair::default()).unwrap();

        create_custom_swap_fee_handler(
            deps.as_mut(),
            info.clone(),
            denom.clone(),
            Decimal::percent(1),
        )
        .unwrap();

        let custom_fees = get_custom_swap_fees_handler(deps.as_ref()).unwrap();

        assert_eq!(custom_fees.len(), 1);
        assert_eq!(custom_fees[0], (denom.clone(), Decimal::percent(1)));

        remove_custom_swap_fee_handler(deps.as_mut(), info, denom).unwrap();

        let custom_fees = get_custom_swap_fees_handler(deps.as_ref()).unwrap();

        assert_eq!(custom_fees.len(), 0);
    }
}
