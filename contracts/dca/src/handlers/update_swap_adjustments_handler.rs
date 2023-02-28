use crate::{
    error::ContractError, state::swap_adjustments::update_swap_adjustments,
    types::dca_plus_config::DCAPlusDirection,
};
use cosmwasm_std::{Decimal, DepsMut, Response};
use fin_helpers::position_type::PositionType;

pub fn update_swap_adjustments_handler(
    deps: DepsMut,
    position_type: PositionType,
    adjustments: Vec<(u8, Decimal)>,
) -> Result<Response, ContractError> {
    update_swap_adjustments(deps.storage, position_type, adjustments)?;
    Ok(Response::new())
}
