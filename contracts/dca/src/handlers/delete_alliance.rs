use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::error::ContractError;
use crate::{state::alliance::delete_alliance, validation_helpers::assert_sender_is_admin};

pub fn delete_alliance_handler(
    deps: DepsMut,
    info: MessageInfo,
    alliance: String,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;

    delete_alliance(deps.storage, alliance.clone())?;

    Ok(Response::new().add_attribute("delete_alliance", alliance))
}
