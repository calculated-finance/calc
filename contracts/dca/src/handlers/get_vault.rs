use crate::{
    msg::VaultResponse,
    state::{trigger_store, vault_store},
};
use cosmwasm_std::{Deps, StdResult, Uint128};

pub fn get_vault(deps: Deps, vault_id: Uint128) -> StdResult<VaultResponse> {
    let vault = vault_store().load(deps.storage, vault_id.into())?;
    let trigger = trigger_store().may_load(deps.storage, vault_id.u128())?;

    Ok(VaultResponse { vault, trigger })
}
