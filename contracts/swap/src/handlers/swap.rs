use crate::errors::contract_error::ContractError;
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal256, DepsMut, QuerierWrapper, SubMsg, WasmMsg,
};
use fin_helpers::swaps::create_fin_swap_message;
use kujira::fin::ExecuteMsg as FINExecuteMsg;

pub fn swap(deps: DepsMut) -> Result<(), ContractError> {
    // make fin swap
    // take fees?
    // invoke callback with funds
    Ok(())
}
