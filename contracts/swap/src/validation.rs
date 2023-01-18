use cosmwasm_std::{Addr, Storage};

use crate::{errors::contract_error::ContractError, state::config::get_config};

pub fn assert_sender_is_admin(
    storage: &mut dyn Storage,
    sender: Addr,
) -> Result<(), ContractError> {
    let config = get_config(storage)?;
    if sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}
