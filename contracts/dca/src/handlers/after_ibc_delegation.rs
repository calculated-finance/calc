use crate::error::ContractError;
use crate::state::cache::CACHE;
use crate::state::events::create_event;
use crate::state::vaults::get_vault;
use base::events::event::{EventBuilder, EventData};
use base::helpers::message_helpers::{
    get_attribute_in_event, get_coin_from_display_formatted_coin,
};
use cosmwasm_std::SubMsgResult;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub fn after_ibc_delegation(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id.into())?;

    match reply.result {
        SubMsgResult::Ok(_) => {
            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block,
                    EventData::DcaVaultIbcDelegationSucceeded {},
                ),
            )?;
        }
        SubMsgResult::Err(_) => {
            create_event(
                deps.storage,
                EventBuilder::new(vault.id, env.block, EventData::DcaVaultDelegationFailed {}),
            )?;
        }
    }

    Ok(Response::new()
        .add_attribute("method", "after_ibc_delegation")
        .add_attribute("vault_id", vault.id.to_string()))
}
