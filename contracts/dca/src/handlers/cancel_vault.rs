use crate::contract::AFTER_FIN_LIMIT_ORDER_RETRACTED_REPLY_ID;
use crate::error::ContractError;
use crate::message_helpers::unstake_from_bow_message;
use crate::msg::ExecuteMsg;
use crate::state::cache::{
    BowCache, Cache, LimitOrderCache, BOW_CACHE, CACHE, LIMIT_ORDER_CACHE, SOURCE_CACHE,
};
use crate::state::events::create_event;
use crate::state::sources::{remove_source, save_source};
use crate::state::triggers::delete_trigger;
use crate::state::vaults::{get_vault, update_vault};
use crate::types::source::Source;
use crate::types::vault::Vault;
use crate::validation_helpers::{
    assert_sender_is_admin_or_vault_owner_or_contract, assert_vault_is_not_cancelled,
};
use base::events::event::{EventBuilder, EventData};
use base::triggers::trigger::TriggerConfiguration;
use base::vaults::vault::VaultStatus;
use cosmwasm_std::{to_binary, BankMsg, Coin, DepsMut, Response, StdError, Uint128, WasmMsg};
use cosmwasm_std::{CosmosMsg, Env, MessageInfo, SubMsg};
use fin_helpers::limit_orders::create_retract_order_sub_msg;
use fin_helpers::queries::query_order_details;

pub fn cancel_vault(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    let vault = get_vault(deps.storage, vault_id)?;

    assert_sender_is_admin_or_vault_owner_or_contract(
        deps.storage,
        &info.sender,
        &vault.owner,
        &env,
    )?;
    assert_vault_is_not_cancelled(&vault)?;

    if let Some(source) = vault.source.clone() {
        let mut sub_messages: Vec<SubMsg> = Vec::new();

        match &source {
            Source::Bow { address } => {
                BOW_CACHE.save(
                    deps.storage,
                    &BowCache {
                        pool_address: address.clone(),
                        lp_token_balance: Some(vault.balance.clone()),
                        deposit: vec![],
                        withdrawal: vec![],
                    },
                )?;

                sub_messages.push(unstake_from_bow_message(deps, env, &vault)?);
            }
        }

        SOURCE_CACHE.save(deps.storage, &source)?;
        remove_source(deps.storage, vault.id)?;

        return Ok(Response::new()
            .add_attribute("method", "cancel_vault")
            .add_attribute("owner", vault.owner.to_string())
            .add_attribute("vault_id", vault_id)
            .add_submessages(sub_messages)
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::CancelVault { vault_id: vault.id })?,
                funds: vec![],
            })));
    }

    create_event(
        deps.storage,
        EventBuilder::new(vault.id, env.block.clone(), EventData::DcaVaultCancelled {}),
    )?;

    if let Some(source) = SOURCE_CACHE.may_load(deps.storage)? {
        save_source(deps.storage, vault.id, source)?;
        SOURCE_CACHE.remove(deps.storage);
    }

    return match vault.trigger {
        Some(TriggerConfiguration::Time { .. }) | None => {
            if vault.trigger.is_some() {
                delete_trigger(deps.storage, vault.id.into())?;
            }
            refund_vault_balance(deps, &vault)
        }
        Some(TriggerConfiguration::FinLimitOrder { order_idx, .. }) => {
            cancel_fin_limit_order_trigger(
                deps,
                env,
                order_idx
                    .expect(format!("order idx for price trigger for vault {}", vault.id).as_str()),
                &vault,
            )
        }
    };
}

fn refund_vault_balance(deps: &mut DepsMut, vault: &Vault) -> Result<Response, ContractError> {
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

    update_vault(deps.storage, vault.id.into(), |stored_vault| {
        if let Some(mut stored_vault) = stored_vault {
            stored_vault.status = VaultStatus::Cancelled;
            stored_vault.balance = Coin::new(0, stored_vault.get_swap_denom());
            return Ok(stored_vault);
        }
        Err(StdError::NotFound {
            kind: format!("Vault {}", vault.id),
        })
    })?;

    Ok(response)
}

fn cancel_fin_limit_order_trigger(
    deps: &mut DepsMut,
    env: &Env,
    order_idx: Uint128,
    vault: &Vault,
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
        swap_denom_balance: deps
            .querier
            .query_balance(&env.contract.address, &vault.get_swap_denom())?,
        receive_denom_balance: deps
            .querier
            .query_balance(&env.contract.address, &vault.get_receive_denom())?,
    };

    CACHE.save(
        deps.storage,
        &Cache {
            vault_id: vault.id,
            owner: vault.owner.clone(),
        },
    )?;

    LIMIT_ORDER_CACHE.save(deps.storage, &limit_order_cache)?;

    let fin_retract_order_sub_msg = create_retract_order_sub_msg(
        vault.pair.address.clone(),
        order_idx,
        AFTER_FIN_LIMIT_ORDER_RETRACTED_REPLY_ID,
    );

    Ok(Response::new()
        .add_attribute("method", "cancel_vault")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id)
        .add_submessage(fin_retract_order_sub_msg))
}
