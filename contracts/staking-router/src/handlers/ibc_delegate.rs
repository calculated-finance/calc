
use cosmwasm_std::Response;
use crate::ContractError;

pub fn ibc_delegate(
    _delegator_address: String,
    _validator_address: String
) -> Result<Response, ContractError> {
    unimplemented!()
}