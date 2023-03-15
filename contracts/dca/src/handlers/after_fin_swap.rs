use crate::error::ContractError;
use crate::helpers::disbursement_helpers::get_disbursement_messages;
use crate::helpers::fee_helpers::{get_delegation_fee_rate, get_fee_messages, get_swap_fee_rate};
use crate::helpers::vault_helpers::get_swap_amount;
use crate::state::cache::{CACHE, SWAP_CACHE};
use crate::state::disburse_escrow_tasks::save_disburse_escrow_task;
use crate::state::events::create_event;
use crate::state::triggers::delete_trigger;
use crate::state::vaults::{get_vault, update_vault};
use base::events::event::{EventBuilder, EventData, ExecutionSkippedReason};
use base::helpers::coin_helpers::add_to_coin;
use base::helpers::math_helpers::checked_mul;
use base::vaults::vault::VaultStatus;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{Attribute, Coin, DepsMut, Env, Reply, Response};
use cosmwasm_std::{Decimal, SubMsg, SubMsgResult};

pub fn after_fin_swap(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let mut vault = get_vault(deps.storage, cache.vault_id.into())?;

    let mut attributes: Vec<Attribute> = Vec::new();
    let mut sub_msgs: Vec<SubMsg> = Vec::new();

    match reply.result {
        SubMsgResult::Ok(_) => {
            let swap_cache = SWAP_CACHE.load(deps.storage)?;

            let swap_denom_balance = &deps
                .querier
                .query_balance(&env.contract.address, &vault.get_swap_denom())?;

            let receive_denom_balance = &deps
                .querier
                .query_balance(&env.contract.address, &vault.get_receive_denom())?;

            let coin_sent = Coin::new(
                (swap_cache.swap_denom_balance.amount - swap_denom_balance.amount).into(),
                swap_denom_balance.denom.clone(),
            );

            let coin_received = Coin::new(
                (receive_denom_balance.amount - swap_cache.receive_denom_balance.amount).into(),
                receive_denom_balance.denom.clone(),
            );

            let swap_fee_rate = match vault.dca_plus_config {
                Some(_) => Decimal::zero(),
                None => get_swap_fee_rate(&deps, &vault)?,
            };

            let automation_fee_rate = match vault.dca_plus_config {
                Some(_) => Decimal::zero(),
                None => get_delegation_fee_rate(&deps, &vault)?,
            };

            let swap_fee = checked_mul(coin_received.amount, swap_fee_rate)?;
            let total_after_swap_fee = coin_received.amount - swap_fee;
            let automation_fee = checked_mul(total_after_swap_fee, automation_fee_rate)?;
            let total_fee = swap_fee + automation_fee;
            let mut total_after_total_fee = coin_received.amount - total_fee;

            sub_msgs.append(&mut get_fee_messages(
                deps.as_ref(),
                env.clone(),
                vec![swap_fee, automation_fee],
                coin_received.denom.clone(),
                false,
            )?);

            vault.balance.amount -= get_swap_amount(&deps.as_ref(), &env, vault.clone())?.amount;

            if !vault.has_sufficient_funds() {
                vault.status = VaultStatus::Inactive;
                delete_trigger(deps.storage, vault.id)?;
            }

            vault.swapped_amount = add_to_coin(vault.swapped_amount, coin_sent.amount)?;
            vault.received_amount = add_to_coin(vault.received_amount, total_after_total_fee)?;

            if let Some(mut dca_plus_config) = vault.dca_plus_config.clone() {
                let amount_to_escrow = total_after_total_fee * dca_plus_config.escrow_level;
                dca_plus_config.escrowed_balance += amount_to_escrow;

                total_after_total_fee -= amount_to_escrow;
                vault.dca_plus_config = Some(dca_plus_config);
            }

            update_vault(deps.storage, &vault)?;

            sub_msgs.append(&mut get_disbursement_messages(
                deps.as_ref(),
                &vault,
                total_after_total_fee,
            )?);

            // if vault.is_active() {
            //     save_trigger(
            //         deps.storage,
            //         Trigger {
            //             vault_id: vault.id,
            //             configuration: TriggerConfiguration::Time {
            //                 target_time: get_next_target_time(
            //                     env.block.time,
            //                     match vault.trigger {
            //                         Some(TriggerConfiguration::Time { target_time }) => target_time,
            //                         _ => env.block.time,
            //                     },
            //                     vault.time_interval.clone(),
            //                 ),
            //             },
            //         },
            //     )?;
            // }

            create_event(
                deps.storage,
                EventBuilder::new(
                    vault.id,
                    env.block.clone(),
                    EventData::DcaVaultExecutionCompleted {
                        sent: coin_sent.clone(),
                        received: coin_received.clone(),
                        fee: Coin::new(total_fee.into(), coin_received.denom.clone()),
                    },
                ),
            )?;

            attributes.push(Attribute::new("status", "success"));
        }
        SubMsgResult::Err(_) => {
            if !vault.has_sufficient_funds() {
                create_event(
                    deps.storage,
                    EventBuilder::new(
                        vault.id,
                        env.block.to_owned(),
                        EventData::DcaVaultExecutionSkipped {
                            reason: ExecutionSkippedReason::UnknownFailure,
                        },
                    ),
                )?;

                vault.status = VaultStatus::Inactive;

                update_vault(deps.storage, &vault)?;
                delete_trigger(deps.storage, vault.id)?;
            } else {
                create_event(
                    deps.storage,
                    EventBuilder::new(
                        vault.id,
                        env.block.to_owned(),
                        EventData::DcaVaultExecutionSkipped {
                            reason: ExecutionSkippedReason::SlippageToleranceExceeded,
                        },
                    ),
                )?;
            }

            attributes.push(Attribute::new("status", "skipped"));
        }
    }

    if vault.is_inactive() {
        if let Some(_) = vault.dca_plus_config {
            save_disburse_escrow_task(
                deps.storage,
                vault.id,
                vault.get_expected_execution_completed_date(env.block.time),
            )?;
        }
    }

    Ok(Response::new()
        .add_attribute("method", "fin_swap_completed")
        .add_attribute("owner", vault.owner.to_string())
        .add_attribute("vault_id", vault.id)
        .add_attributes(attributes)
        .add_submessages(sub_msgs))
}
