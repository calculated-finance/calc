use cosmwasm_std::{Deps, StdResult};

use crate::state::{State, STATE};

pub fn get_state(deps: Deps) -> StdResult<State> {
    let state = STATE.load(deps.storage)?;
    Ok(state)
}
