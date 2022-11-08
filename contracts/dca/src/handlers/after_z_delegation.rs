use crate::error::ContractError;
use crate::state::cache::CACHE;
use crate::state::config::get_config;
use crate::state::events::create_event;
use crate::state::vaults::get_vault;
use base::events::event::{EventBuilder, EventData};
use base::helpers::math_helpers::checked_mul;
use base::helpers::message_helpers::{
    get_attribute_in_event, get_coin_from_display_formatted_coin,
};
use cosmwasm_std::{SubMsgResult, Coin, Uint128, CosmosMsg, BankMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub fn after_z_delegation(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id.into())?;
    let mut response = Response::new()
        .add_attribute("method", "after_z_delegation")
        .add_attribute("vault_id", vault.id.to_string());

    match reply.result {
        SubMsgResult::Ok(_) => {
            let z_delegate_response = reply.result.into_result().unwrap();

            let validator_address =
                get_attribute_in_event(&z_delegate_response.events, "delegate", "validator")?;

            let display_formatted_coin =
                get_attribute_in_event(&z_delegate_response.events, "delegate", "amount")?;

            let delegation = get_coin_from_display_formatted_coin(display_formatted_coin);

            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block,
                    EventData::DcaVaultZDelegationSucceeded {
                        validator_address,
                        delegation,
                    },
                ),
            )?;
        }
        SubMsgResult::Err(_) => {

            // it's possible some delegation messages succeeed
            // which is fine because this is jsut fee taking
            // need to refund exactly once and make sure we dont refund from stale cache
            // check greater than 0
            // how do we overwrite this
            if let Some(refunded) = cache.refunded {
                if !refunded {



                }
            }

            create_event(
                deps.storage,
                EventBuilder::new(vault.id, env.block, EventData::DcaVaultDelegationFailed {}),
            )?;
        }
    }

    Ok(response)
}
