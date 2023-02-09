use crate::{
    contract::ContractResult, errors::contract_error::ContractError, state::config::get_config,
};
use cosmwasm_std::{Addr, Coin, Env, Storage, Uint128};

pub fn assert_sender_is_admin(storage: &mut dyn Storage, sender: Addr) -> ContractResult<()> {
    let config = get_config(storage)?;
    if sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn assert_sender_is_contract(sender: &Addr, env: &Env) -> ContractResult<()> {
    if sender != &env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn assert_exactly_one_asset(funds: &Vec<Coin>) -> ContractResult<()> {
    if funds.is_empty() || funds.len() > 1 {
        return Err(ContractError::CustomError {
            val: format!("received {} denoms but required exactly 1", funds.len()),
        });
    }
    Ok(())
}

pub fn assert_swap_amount_is_greater_than_50000(swap_amount: Uint128) -> Result<(), ContractError> {
    if swap_amount <= Uint128::from(50000u128) {
        return Err(ContractError::CustomError {
            val: String::from("swap amount must be greater than 50000"),
        });
    }
    Ok(())
}
