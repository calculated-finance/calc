use crate::validation_helpers::assert_sender_is_admin;
use crate::{error::ContractError, state::PAIRS};
use base::pair::Pair;
use cosmwasm_std::{Addr, DepsMut};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Env, MessageInfo, Response};

pub fn create_pair(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    base_denom: String,
    quote_denom: String,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.as_ref(), info.sender)?;

    let validated_pair_address: Addr = deps.api.addr_validate(&address)?;

    let pair: Pair = Pair {
        address: validated_pair_address.clone(),
        base_denom: base_denom.clone(),
        quote_denom: quote_denom.clone(),
    };

    let existing_pair = PAIRS.may_load(deps.storage, validated_pair_address.clone())?;
    match existing_pair {
        Some(_pair) => Err(ContractError::CustomError {
            val: String::from("pair already exists at given address"),
        }),
        None => {
            PAIRS.save(deps.storage, validated_pair_address.clone(), &pair)?;
            Ok(Response::new()
                .add_attribute("method", "create_pair")
                .add_attribute("address", validated_pair_address.to_string())
                .add_attribute("base_denom", base_denom)
                .add_attribute("quote_denom", quote_denom))
        }
    }
}