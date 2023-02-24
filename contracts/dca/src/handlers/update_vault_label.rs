use crate::{
    error::ContractError,
    state::vaults::{get_vault, update_vault},
    validation_helpers::{assert_vault_is_not_cancelled, asset_sender_is_vault_owner},
};
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

pub fn update_vault_label(
    deps: DepsMut,
    info: MessageInfo,
    vault_id: Uint128,
    label: Option<String>,
) -> Result<Response, ContractError> {
    let mut vault = get_vault(deps.storage, vault_id)?;

    assert_vault_is_not_cancelled(&vault)?;

    if let Some(label) = label {
        vault.label = Some(label);
    }

    let updated_vault = update_vault(deps.storage, &vault)?;

    asset_sender_is_vault_owner(updated_vault.owner, info.sender)?;

    Ok(Response::default().add_attribute("method", "update_vault"))
}
