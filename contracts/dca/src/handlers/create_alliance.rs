use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::error::ContractError;
use crate::{state::alliance::create_alliance, validation_helpers::assert_sender_is_admin};

pub fn create_alliance_handler(
    deps: DepsMut,
    info: MessageInfo,
    alliance: String,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;

    create_alliance(deps.storage, alliance.clone())?;

    Ok(Response::new().add_attribute("create_alliance", alliance))
}
