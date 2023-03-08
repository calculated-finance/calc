use crate::contract::AFTER_FIN_LIMIT_ORDER_RETRACTED_REPLY_ID;
use crate::error::ContractError;
use crate::helpers::message_helpers::create_claim_escrowed_funds_message;
use crate::helpers::validation_helpers::{
    assert_sender_is_admin_or_vault_owner, assert_vault_is_not_cancelled,
};
use crate::state::cache::{Cache, LimitOrderCache, CACHE, LIMIT_ORDER_CACHE};
use crate::state::events::create_event;
use crate::state::triggers::delete_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::types::vault::Vault;
use base::events::event::{EventBuilder, EventData};
use base::triggers::trigger::TriggerConfiguration;
use base::vaults::vault::VaultStatus;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{BankMsg, DepsMut, Response, Uint128};
use cosmwasm_std::{Coin, CosmosMsg, Env, MessageInfo, SubMsg};
use fin_helpers::limit_orders::create_retract_order_sub_msg;
use fin_helpers::queries::query_order_details;

pub fn cancel_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    let mut vault = get_vault(deps.storage, vault_id)?;

    assert_sender_is_admin_or_vault_owner(deps.storage, vault.owner.clone(), info.sender.clone())?;
    assert_vault_is_not_cancelled(&vault)?;

    create_event(
        deps.storage,
        EventBuilder::new(vault.id, env.block.clone(), EventData::DcaVaultCancelled {}),
    )?;

    let cancel_trigger_message = create_cancel_fin_trigger_message(deps, &mut vault, &env)?;
    let claim_escrowed_funds_message = create_claim_escrowed_funds_message(&vault, env);

    Ok(Response::new()
        .add_attribute("method", "cancel_vault")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id)
        .add_submessages(
            vec![cancel_trigger_message, claim_escrowed_funds_message]
                .into_iter()
                .flatten()
                .collect::<Vec<SubMsg>>(),
        ))
}

fn create_cancel_fin_trigger_message(
    deps: DepsMut,
    vault: &mut Vault,
    env: &Env,
) -> Result<Option<SubMsg>, ContractError> {
    Ok(match vault.trigger {
        Some(TriggerConfiguration::Time { .. }) | None => {
            if let Some(_) = vault.trigger {
                delete_trigger(deps.storage, vault.id.into())?;
            }

            let refund_vault_balance_message = if vault.balance.amount.gt(&Uint128::zero()) {
                Some(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                    to_address: vault.owner.to_string(),
                    amount: vec![vault.balance.clone()],
                })))
            } else {
                None
            };

            vault.status = VaultStatus::Cancelled;
            vault.balance = Coin::new(0, vault.get_swap_denom());
            update_vault(deps.storage, &*vault)?;

            Ok(refund_vault_balance_message)
        }
        Some(TriggerConfiguration::FinLimitOrder { order_idx, .. }) => order_idx.map_or(
            Err(ContractError::CustomError {
                val: "Fin limit order does not have an order_idx".to_string(),
            }),
            |order_idx| {
                let limit_order_details =
                    query_order_details(deps.querier, vault.pair.address.clone(), order_idx)?;

                let limit_order_cache = LimitOrderCache {
                    order_idx,
                    offer_amount: limit_order_details.offer_amount,
                    original_offer_amount: limit_order_details.original_offer_amount,
                    filled: limit_order_details.filled_amount,
                    quote_price: limit_order_details.quote_price,
                    created_at: limit_order_details.created_at,
                    swap_denom_balance: deps
                        .querier
                        .query_balance(&env.contract.address, &vault.get_swap_denom())?,
                    receive_denom_balance: deps
                        .querier
                        .query_balance(&env.contract.address, &vault.get_receive_denom())?,
                };

                LIMIT_ORDER_CACHE.save(deps.storage, &limit_order_cache)?;

                let fin_retract_order_sub_msg = create_retract_order_sub_msg(
                    vault.pair.address.clone(),
                    order_idx,
                    AFTER_FIN_LIMIT_ORDER_RETRACTED_REPLY_ID,
                );

                let cache = Cache {
                    vault_id: vault.id,
                    owner: vault.owner.clone(),
                };

                CACHE.save(deps.storage, &cache)?;

                Ok(Some(fin_retract_order_sub_msg))
            },
        ),
    }?)
}
