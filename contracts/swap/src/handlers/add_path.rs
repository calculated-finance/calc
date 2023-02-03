use crate::{state::paths::add_path, types::exchange::Pair};
use cosmwasm_std::{DepsMut, Response, StdResult};

pub fn add_path_handler(deps: DepsMut, denoms: [String; 2], exchange: Pair) -> StdResult<Response> {
    add_path(deps.storage, denoms.clone(), exchange.clone())?;
    Ok(Response::new()
        .add_attribute("method", "add_path")
        .add_attribute("denoms", format!("[{}, {}]", denoms[0], denoms[1]))
        .add_attribute("pair", format!("{:?}", exchange)))
}
