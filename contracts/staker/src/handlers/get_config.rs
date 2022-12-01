
use cosmwasm_std::{Deps, StdResult};

use crate::state::{Config, CONFIG};

pub fn get_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}