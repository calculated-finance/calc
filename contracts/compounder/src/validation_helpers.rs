use cosmwasm_std::Coin;

use crate::ContractError;

pub fn assert_exactly_one_asset(funds: Vec<Coin>) -> Result<(), ContractError> {
    if funds.is_empty() || funds.len() > 1 {
        Err(ContractError::CustomError {
            val: format!("received {} denoms but required exactly 1", funds.len()),
        })
    } else {
        Ok(())
    }
}

pub fn assert_denom_is_kuji(funds: Vec<Coin>) -> Result<(), ContractError> {
    if funds[0].denom != String::from("ukuji") {
        Err(ContractError::CustomError {
            val: String::from("only ukuji can be delegated"),
        })
    } else {
        Ok(())
    }
}
