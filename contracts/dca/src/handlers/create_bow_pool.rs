use crate::state::pools::get_pool;
use crate::validation_helpers::assert_sender_is_admin;
use crate::{error::ContractError, state::pools::save_pool};
use base::pool::Pool;
use cosmwasm_std::{Addr, DepsMut};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{MessageInfo, Response};

pub fn create_bow_pool(
    deps: DepsMut,
    info: &MessageInfo,
    address: &Addr,
    denoms: [String; 2],
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender.clone())?;

    deps.api.addr_validate(&address.to_string())?;

    let pool = Pool {
        address: address.clone(),
        denoms,
    };

    let existing_pool = get_pool(deps.storage, address)?;

    match existing_pool {
        Some(_) => Err(ContractError::CustomError {
            val: format!("Bow Pool {} already exists", address.to_string()),
        }),
        None => {
            save_pool(deps.storage, &pool)?;

            Ok(Response::new()
                .add_attribute("method", "create_pool")
                .add_attribute("address", pool.address.to_string())
                .add_attribute("denoms", format!("{:?}", pool.denoms)))
        }
    }
}

#[cfg(test)]
mod create_bow_pool_tests {
    use crate::{
        state::pools::get_pool,
        tests::{helpers::instantiate_contract, mocks::ADMIN},
    };
    use base::pool::Pool;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    use super::create_bow_pool;

    #[test]
    pub fn creates_new_pool() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env, info.clone());

        let address = Addr::unchecked("bow-pool");
        let denoms = ["ukuji".to_string(), "udemo".to_string()];

        create_bow_pool(deps.as_mut(), &info, &address, denoms.clone()).unwrap();

        let pool = get_pool(deps.as_mut().storage, &address).unwrap();

        assert_eq!(pool, Some(Pool { address, denoms }))
    }

    #[test]
    pub fn fails_when_pool_already_exists() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env, info.clone());

        let address = Addr::unchecked("bow-pool");
        let denoms = ["ukuji".to_string(), "udemo".to_string()];

        create_bow_pool(deps.as_mut(), &info, &address, denoms.clone()).unwrap();
        let result = create_bow_pool(deps.as_mut(), &info, &address, denoms.clone()).unwrap_err();

        assert_eq!(
            result.to_string(),
            format!("Error: Bow Pool {} already exists", address.to_string())
        )
    }

    #[test]
    pub fn returns_unauthorised_when_called_by_non_admin() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let mut info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env, info.clone());

        info.sender = Addr::unchecked("not-admin");

        let address = Addr::unchecked("bow-pool");
        let denoms = ["ukuji".to_string(), "udemo".to_string()];

        let result = create_bow_pool(deps.as_mut(), &info, &address, denoms.clone()).unwrap_err();

        assert_eq!(result.to_string(), "Unauthorized")
    }
}
