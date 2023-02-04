use crate::{state::paths::add_path, types::pair::Pair, validation::assert_sender_is_admin};
use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

pub fn add_pair_handler(deps: DepsMut, info: MessageInfo, pair: Pair) -> StdResult<Response> {
    assert_sender_is_admin(deps.storage, info.sender)?;

    add_path(deps.storage, pair.clone())?;

    Ok(Response::new()
        .add_attribute("method", "add_pair")
        .add_attribute("pair", format!("{:?}", pair)))
}
