use crate::msg::ConfigResponse;
use crate::state::config::get_config as fetch_config;
use cosmwasm_std::{Deps, StdResult};

pub fn get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = fetch_config(deps.storage)?;
    Ok(ConfigResponse { config })
}
