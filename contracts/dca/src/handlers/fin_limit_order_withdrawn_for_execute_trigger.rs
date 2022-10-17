use crate::constants::ONE_HUNDRED;
use crate::error::ContractError;
use crate::state::{save_event, save_trigger, vault_store, CACHE, CONFIG, LIMIT_ORDER_CACHE};
use crate::vault::Vault;
use base::events::event::{EventBuilder, EventData};
use base::helpers::time_helpers::get_next_target_time;
use base::triggers::trigger::{Trigger, TriggerConfiguration};
use base::vaults::vault::VaultStatus;
use cosmwasm_std::Env;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{BankMsg, Coin, DepsMut, Reply, Response};

pub fn fin_limit_order_withdrawn_for_execute_vault(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let limit_order_cache = LIMIT_ORDER_CACHE.load(deps.storage)?;
    let vault = vault_store().load(deps.storage, cache.vault_id.into())?;

    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            save_trigger(
                deps.storage,
                Trigger {
                    vault_id: vault.id,
                    configuration: TriggerConfiguration::Time {
                        target_time: get_next_target_time(
                            env.block.time,
                            env.block.time,
                            vault.time_interval.clone(),
                        ),
                    },
                },
            )?;

            vault_store().update(
                deps.storage,
                vault.id.into(),
                |vault| -> Result<Vault, ContractError> {
                    match vault {
                        Some(mut existing_vault) => {
                            existing_vault.balance.amount -=
                                limit_order_cache.original_offer_amount;
                            if existing_vault.low_funds() {
                                existing_vault.status = VaultStatus::Inactive
                            }

                            Ok(existing_vault)
                        }
                        None => Err(ContractError::CustomError {
                            val: format!(
                                "could not find vault for address: {} with id: {}",
                                cache.owner, cache.vault_id
                            ),
                        }),
                    }
                },
            )?;

            let coin_received_from_limit_order = Coin {
                denom: vault.get_receive_denom().clone(),
                amount: limit_order_cache.filled,
            };

            let config = CONFIG.load(deps.storage)?;

            let execution_fee = Coin::new(
                (coin_received_from_limit_order
                    .amount
                    .checked_multiply_ratio(config.fee_percent, ONE_HUNDRED)?)
                .u128(),
                &coin_received_from_limit_order.denom,
            );

            let funds_to_redistribute = Coin::new(
                (coin_received_from_limit_order.amount - execution_fee.amount).u128(),
                &coin_received_from_limit_order.denom,
            );

            let funds_redistribution_bank_msg: BankMsg = BankMsg::Send {
                to_address: vault.owner.to_string(),
                amount: vec![funds_to_redistribute],
            };

            let fee_collector_bank_msg: BankMsg = BankMsg::Send {
                to_address: config.fee_collector.to_string(),
                amount: vec![execution_fee.clone()],
            };

            save_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block,
                    EventData::DCAVaultExecutionCompleted {
                        sent: Coin {
                            denom: vault.get_swap_denom().clone(),
                            amount: limit_order_cache.original_offer_amount,
                        },
                        received: coin_received_from_limit_order,
                        fee: execution_fee,
                    },
                ),
            )?;

            LIMIT_ORDER_CACHE.remove(deps.storage);
            CACHE.remove(deps.storage);

            Ok(Response::new()
                .add_attribute(
                    "method",
                    "after_fin_limit_order_withdrawn_for_execute_trigger",
                )
                .add_attribute("vault_id", vault.id)
                .add_message(funds_redistribution_bank_msg)
                .add_message(fee_collector_bank_msg))
        }
        cosmwasm_std::SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!(
                "failed to withdraw fin limit order for vault id: {} - {}",
                vault.id, e
            ),
        }),
    }
}
