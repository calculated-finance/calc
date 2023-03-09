use crate::{msg::ClaimEscrowTasksResponse, state::claim_escrow_tasks::get_claim_escrow_tasks};
use cosmwasm_std::{Deps, Env, StdResult};

pub fn get_claim_escrow_tasks_handler(deps: Deps, env: Env) -> StdResult<ClaimEscrowTasksResponse> {
    let tasks = get_claim_escrow_tasks(deps.storage, env.block.time)?;

    Ok(ClaimEscrowTasksResponse { vault_ids: tasks })
}
