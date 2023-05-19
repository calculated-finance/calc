use crate::state::old_vaults::get_vaults_by_address as fetch_vaults_by_address;
use crate::{helpers::validation_helpers::assert_page_limit_is_valid, msg::VaultsResponse};
use base::vaults::vault::OldVaultStatus;
use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};

pub fn get_vaults_by_address(
    deps: Deps,
    env: Env,
    address: Addr,
    status: Option<OldVaultStatus>,
    start_after: Option<Uint128>,
    limit: Option<u16>,
) -> StdResult<VaultsResponse> {
    deps.api.addr_validate(&address.to_string())?;
    assert_page_limit_is_valid(deps.storage, limit)?;

    let vaults = fetch_vaults_by_address(deps.storage, env, address, status, start_after, limit)?;

    Ok(VaultsResponse { vaults })
}
