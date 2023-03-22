use crate::helpers::validation_helpers::assert_sender_is_admin;
use crate::state::pairs::PAIRS;
use crate::{error::ContractError, state::pools::POOLS};
use base::pair::Pair;
use cosmwasm_std::{Addr, DepsMut};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Env, MessageInfo, Response};
use osmosis_helpers::pool::Pool;
pub fn create_pool(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pool_id: u64,
    base_denom: String,
    quote_denom: String,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;

    //deps.api.addr_validate(&pool_id.to_string())?;

    let pool: Pool = Pool {
        pool_id: pool_id.clone(),
        base_denom: base_denom.clone(),
        quote_denom: quote_denom.clone(),
    };

    let existing_pool = POOLS.may_load(deps.storage, pool_id.clone())?;
    match existing_pool {
        Some(_) => Err(ContractError::CustomError {
            val: String::from("pool already exists at given address"),
        }),
        None => {
            POOLS.save(deps.storage, pool_id.clone(), &pool)?;
            Ok(Response::new()
                .add_attribute("method", "create_pair")
                .add_attribute("pool_id", pool_id.to_string())
                .add_attribute("base_denom", base_denom)
                .add_attribute("quote_denom", quote_denom))
        }
    }
}
