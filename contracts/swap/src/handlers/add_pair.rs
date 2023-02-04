use crate::{state::paths::add_path, types::pair::Pair};
use cosmwasm_std::{DepsMut, Response, StdResult};

pub fn add_pair_handler(deps: DepsMut, pair: Pair) -> StdResult<Response> {
    add_path(deps.storage, pair.clone())?;
    Ok(Response::new()
        .add_attribute("method", "add_path")
        .add_attribute("pair", format!("{:?}", pair)))
}
