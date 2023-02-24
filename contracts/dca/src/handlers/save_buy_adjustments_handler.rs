use crate::{error::ContractError, state::buy_adjustments::save_buy_adjustments};
use cosmwasm_std::{Decimal, DepsMut, Response};

pub fn save_buy_adjustments_handler(
    deps: DepsMut,
    adjustments: Vec<(u8, Decimal)>,
) -> Result<Response, ContractError> {
    save_buy_adjustments(deps.storage, adjustments)?;
    Ok(Response::new())
}
