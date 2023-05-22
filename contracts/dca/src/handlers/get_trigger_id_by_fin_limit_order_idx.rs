use crate::{msg::TriggerIdResponse, state::triggers::trigger_store};
#[cfg(not(feature = "library"))]
use cosmwasm_std::Uint128;
use cosmwasm_std::{Deps, StdError, StdResult};

pub fn get_trigger_id_by_fin_limit_order_idx_handler(
    deps: Deps,
    order_idx: Uint128,
) -> StdResult<TriggerIdResponse> {
    let trigger_id = trigger_store()
        .idx
        .order_idx
        .item(deps.storage, order_idx.into())?
        .map(|(_, trigger)| trigger.vault_id);

    if let Some(trigger_id) = trigger_id {
        Ok(TriggerIdResponse { trigger_id })
    } else {
        Err(StdError::generic_err("Trigger not found"))
    }
}
