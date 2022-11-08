use crate::error::ContractError;
use crate::state::cache::{Cache, CACHE};
use crate::state::events::create_event;
use crate::state::vaults::get_vault;
use base::events::event::{EventBuilder, EventData};
use base::helpers::message_helpers::{
    get_attribute_in_event, get_coin_from_display_formatted_coin,
};
use cosmwasm_std::{BankMsg, SubMsgResult};
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
            // which is fine because this is just fee taking
            // need to refund exactly once and make sure we dont refund from stale cache
            // check greater than 0
            if let (Some(refunded), Some(total_automation_fees)) =
                (cache.refunded.clone(), cache.total_automation_fees.clone())
            {
                match (refunded, total_automation_fees) {
                    (false, automation_fees_to_refund) => {
                        response = response.add_message(BankMsg::Send {
                            to_address: cache.owner.to_string(),
                            amount: vec![automation_fees_to_refund],
                        });

                        CACHE.save(
                            deps.storage,
                            &Cache {
                                refunded: Some(true),
                                ..cache.clone()
                            },
                        )?;
                    }
                    _ => {}
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
