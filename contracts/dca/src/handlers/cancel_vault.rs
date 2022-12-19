use crate::contract::{
    AFTER_FIN_LIMIT_ORDER_RETRACTED_REPLY_ID,
    AFTER_FIN_LIMIT_ORDER_WITHDRAWN_FOR_CANCEL_VAULT_REPLY_ID,
};
use crate::error::ContractError;
use crate::state::cache::{Cache, LimitOrderCache, CACHE, LIMIT_ORDER_CACHE};
use crate::state::events::create_event;
use crate::state::fin_limit_order_change_timestamp::FIN_LIMIT_ORDER_CHANGE_TIMESTAMP;
use crate::state::triggers::delete_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::types::vault::Vault;
use crate::validation_helpers::{
    assert_sender_is_admin_or_vault_owner, assert_vault_is_not_cancelled,
};
use base::events::event::{EventBuilder, EventData};
use base::helpers::message_helpers::get_attribute_in_event;
use base::triggers::trigger::TriggerConfiguration;
use base::vaults::vault::VaultStatus;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{BankMsg, DepsMut, Response, Uint128};
use cosmwasm_std::{Coin, CosmosMsg, Env, MessageInfo, Reply, StdError, StdResult, SubMsgResult};
use fin_helpers::limit_orders::{
    create_retract_order_sub_msg, create_withdraw_limit_order_sub_msg,
};
use fin_helpers::queries::query_order_details;

pub fn cancel_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    let vault = get_vault(deps.storage, vault_id)?;

    assert_sender_is_admin_or_vault_owner(deps.storage, vault.owner.clone(), info.sender.clone())?;
    assert_vault_is_not_cancelled(&vault)?;

    create_event(
        deps.storage,
        EventBuilder::new(vault.id, env.block, EventData::DcaVaultCancelled {}),
    )?;

    match vault.trigger {
        Some(TriggerConfiguration::Time { .. }) => {
            delete_trigger(deps.storage, vault.id.into())?;
            refund_vault_balance(deps, vault)
        }
        Some(TriggerConfiguration::FinLimitOrder { order_idx, .. }) => {
            cancel_fin_limit_order_trigger(
                deps,
                order_idx
                    .expect(format!("order idx for price trigger for vault {}", vault.id).as_str()),
                vault,
            )
        }
        None => refund_vault_balance(deps, vault),
    }
}

fn refund_vault_balance(deps: DepsMut, vault: Vault) -> Result<Response, ContractError> {
    let mut response = Response::new()
        .add_attribute("method", "cancel_vault")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id);

    if vault.balance.amount.gt(&Uint128::zero()) {
        response = response.add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: vault.owner.to_string(),
            amount: vec![vault.balance.clone()],
        }))
    }

    update_vault(
        deps.storage,
        vault.id.into(),
        |existing_vault| -> StdResult<Vault> {
            match existing_vault {
                Some(mut existing_vault) => {
                    existing_vault.status = VaultStatus::Cancelled;
                    existing_vault.balance = Coin::new(0, existing_vault.get_swap_denom());
                    Ok(existing_vault)
                }
                None => Err(StdError::NotFound {
                    kind: format!("vault for address: {} with id: {}", vault.owner, vault.id),
                }),
            }
        },
    )?;

    Ok(response)
}

fn cancel_fin_limit_order_trigger(
    deps: DepsMut,
    order_idx: Uint128,
    vault: Vault,
) -> Result<Response, ContractError> {
    let limit_order_details =
        query_order_details(deps.querier, vault.pair.address.clone(), order_idx)?;

    let limit_order_cache = LimitOrderCache {
        order_idx,
        offer_amount: limit_order_details.offer_amount,
        original_offer_amount: limit_order_details.original_offer_amount,
        filled: limit_order_details.filled_amount,
        quote_price: limit_order_details.quote_price,
        created_at: limit_order_details.created_at,
    };

    LIMIT_ORDER_CACHE.save(deps.storage, &limit_order_cache)?;

    let fin_retract_order_sub_msg = create_retract_order_sub_msg(
        vault.pair.address,
        order_idx,
        AFTER_FIN_LIMIT_ORDER_RETRACTED_REPLY_ID,
    );

    let cache = Cache {
        vault_id: vault.id,
        owner: vault.owner.clone(),
    };

    CACHE.save(deps.storage, &cache)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_vault")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id)
        .add_submessage(fin_retract_order_sub_msg))
}

pub fn after_fin_limit_order_retracted(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id)?;
    let mut response = Response::new().add_attribute("method", "fin_limit_order_retracted");

    match reply.result {
        SubMsgResult::Ok(_) => {
            let limit_order_cache = LIMIT_ORDER_CACHE.load(deps.storage)?;

            let fin_retract_order_response = reply.result.into_result().unwrap();

            let amount_retracted =
                get_attribute_in_event(&fin_retract_order_response.events, "wasm", "amount")?
                    .parse::<Uint128>()
                    .expect("limit order retracted amount");

            let fin_limit_order_change_timestamp =
                FIN_LIMIT_ORDER_CHANGE_TIMESTAMP.may_load(deps.storage)?;

            let is_new_fin_limit_order = fin_limit_order_change_timestamp.is_some()
                && limit_order_cache.created_at > fin_limit_order_change_timestamp.unwrap();

            // if the entire amount isnt retracted, order was partially filled need to send the partially filled assets to user
            if amount_retracted != limit_order_cache.original_offer_amount {
                let swap_denom_to_return = Coin {
                    denom: vault.get_swap_denom().clone(),
                    amount: if is_new_fin_limit_order {
                        vault.balance.amount + amount_retracted
                    } else {
                        vault.balance.amount - vault.get_swap_amount().amount + amount_retracted
                    },
                };

                if swap_denom_to_return.amount.gt(&Uint128::zero()) {
                    response = response.add_message(CosmosMsg::Bank(BankMsg::Send {
                        to_address: vault.owner.to_string(),
                        amount: vec![swap_denom_to_return.clone()],
                    }));
                }

                let fin_withdraw_sub_msg = create_withdraw_limit_order_sub_msg(
                    vault.pair.address.clone(),
                    limit_order_cache.order_idx,
                    AFTER_FIN_LIMIT_ORDER_WITHDRAWN_FOR_CANCEL_VAULT_REPLY_ID,
                );

                Ok(response
                    .add_attribute("withdraw_required", "true")
                    .add_submessage(fin_withdraw_sub_msg))
            } else {
                if is_new_fin_limit_order {
                    response = response.add_message(CosmosMsg::Bank(BankMsg::Send {
                        to_address: vault.owner.to_string(),
                        amount: vec![Coin::new(amount_retracted.into(), vault.get_swap_denom())],
                    }));
                }

                if vault.balance.amount.gt(&Uint128::zero()) {
                    response = response.add_message(CosmosMsg::Bank(BankMsg::Send {
                        to_address: vault.owner.to_string(),
                        amount: vec![vault.balance.clone()],
                    }));
                }

                update_vault(
                    deps.storage,
                    vault.id.into(),
                    |existing_vault| -> StdResult<Vault> {
                        match existing_vault {
                            Some(mut existing_vault) => {
                                existing_vault.status = VaultStatus::Cancelled;
                                existing_vault.balance =
                                    Coin::new(0, existing_vault.get_swap_denom());
                                Ok(existing_vault)
                            }
                            None => Err(StdError::NotFound {
                                kind: format!(
                                    "vault for address: {} with id: {}",
                                    vault.owner, vault.id
                                ),
                            }),
                        }
                    },
                )?;

                delete_trigger(deps.storage, vault.id)?;

                Ok(response.add_attribute("withdraw_required", "false"))
            }
        }
        SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!(
                "failed to retract fin limit order for vault id: {} - {}",
                vault.id, e
            ),
        }),
    }
}

pub fn after_fin_limit_order_withdrawn_for_cancel_vault(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id.into())?;
    match reply.result {
        SubMsgResult::Ok(_) => {
            let withdraw_order_response = reply.result.into_result().unwrap();

            let received_amount =
                get_attribute_in_event(&withdraw_order_response.events, "transfer", "amount")?
                    .trim_end_matches(&vault.get_receive_denom().to_string())
                    .parse::<Uint128>()
                    .expect("limit order withdrawn amount");

            let coin_received = Coin {
                denom: vault.get_receive_denom().clone(),
                amount: received_amount,
            };

            let mut response = Response::new()
                .add_attribute("method", "fin_limit_order_withdrawn_for_cancel_vault");

            if coin_received.amount.gt(&Uint128::zero()) {
                response = response.add_message(CosmosMsg::Bank(BankMsg::Send {
                    to_address: vault.owner.to_string(),
                    amount: vec![coin_received],
                }));
            }

            update_vault(
                deps.storage,
                vault.id.into(),
                |existing_vault| -> StdResult<Vault> {
                    match existing_vault {
                        Some(mut existing_vault) => {
                            existing_vault.status = VaultStatus::Cancelled;
                            existing_vault.balance = Coin::new(0, existing_vault.get_swap_denom());
                            Ok(existing_vault)
                        }
                        None => Err(StdError::NotFound {
                            kind: format!(
                                "vault for address: {} with id: {}",
                                vault.owner, vault.id
                            ),
                        }),
                    }
                },
            )?;

            delete_trigger(deps.storage, vault.id.into())?;

            Ok(response)
        }
        SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!(
                "failed to withdraw fin limit order for vault id: {} - {}",
                vault.id, e
            ),
        }),
    }
}
