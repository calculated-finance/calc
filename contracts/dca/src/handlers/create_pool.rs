use crate::state::pools::get_pool;
use crate::validation_helpers::{assert_exactly_2_denoms, assert_sender_is_admin};
use crate::{error::ContractError, state::pools::save_pool};
use base::pool::Pool;
use cosmwasm_std::{Addr, DepsMut};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{MessageInfo, Response};

pub fn add_bow_pool(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
    denoms: Vec<String>,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;
    assert_exactly_2_denoms(denoms.clone())?;

    deps.api.addr_validate(&address.to_string())?;

    let pool = Pool {
        address: address.clone(),
        denoms,
    };

    let existing_pool = get_pool(deps.storage, address)?;

    match existing_pool {
        Some(_) => Err(ContractError::CustomError {
            val: String::from("pool already exists at given address"),
        }),
        None => {
            save_pool(deps.storage, pool.address.clone(), &pool)?;

            Ok(Response::new()
                .add_attribute("method", "create_pool")
                .add_attribute("address", pool.address.to_string())
                .add_attribute("denoms", format!("{:?}", pool.denoms)))
        }
    }
}
