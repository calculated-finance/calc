use crate::{
    error::ContractError,
    helpers::{
        disbursement::get_disbursement_messages,
        fees::{get_dca_plus_performance_fee, get_fee_messages},
        validation::assert_sender_is_executor,
    },
    state::{
        disburse_escrow_tasks::delete_disburse_escrow_task,
        events::create_event,
        old_vaults::{get_old_vault, update_old_vault},
    },
    types::dca_plus_config::DcaPlusConfig,
};
use base::{
    events::event::{EventBuilder, EventData},
    helpers::coin_helpers::{empty_of, subtract},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};
use fin_helpers::queries::query_belief_price;

pub fn disburse_escrow_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    assert_sender_is_executor(deps.storage, &env, &info.sender)?;

    let mut vault = get_old_vault(deps.storage, vault_id)?;

    if vault.dca_plus_config.is_none() {
        return Err(ContractError::CustomError {
            val: "Vault is not a DCA+ vault".to_string(),
        });
    }

    let dca_plus_config = vault.dca_plus_config.clone().unwrap();

    let current_price = query_belief_price(&deps.querier, &vault.pair, &vault.get_swap_denom())?;

    let performance_fee = get_dca_plus_performance_fee(&vault, current_price)?;
    let amount_to_disburse = subtract(&dca_plus_config.escrowed_balance, &performance_fee)?;

    vault.dca_plus_config = Some(DcaPlusConfig {
        escrowed_balance: empty_of(dca_plus_config.escrowed_balance),
        ..dca_plus_config
    });

    update_old_vault(deps.storage, &vault)?;
=======
        coin::{empty_of, subtract},
        disbursement::get_disbursement_messages,
        fees::{get_fee_messages, get_performance_fee},
        price::query_belief_price,
        validation::assert_sender_is_executor,
    },
    state::{
        disburse_escrow_tasks::{delete_disburse_escrow_task, get_disburse_escrow_task_due_date},
        events::create_event,
        pairs::find_pair,
        vaults::{get_vault, update_vault},
    },
    types::{
        event::{EventBuilder, EventData},
        vault::Vault,
    },
};
use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn disburse_escrow_handler(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    assert_sender_is_executor(deps.storage, env, &info.sender)?;

    let vault = get_vault(deps.storage, vault_id)?;

    let response = Response::new()
        .add_attribute("disburse_escrow", "true")
        .add_attribute("vault_id", vault.id);

    if vault.escrowed_amount.amount.is_zero() {
        return Ok(response
            .add_attribute(
                "performance_fee",
                format!("{:?}", Coin::new(0, vault.target_denom.clone())),
            )
            .add_attribute(
                "escrow_disbursed",
                format!("{:?}", Coin::new(0, vault.target_denom)),
            ));
    }

    let due_date = get_disburse_escrow_task_due_date(deps.storage, vault.id)?;

    if let Some(due_date) = due_date {
        if env.block.time < due_date {
            return Err(ContractError::CustomError {
                val: "Escrow is not available to be disbursed yet".to_string(),
            });
        }
    }

    let pair = find_pair(deps.storage, vault.denoms())?;
    let current_price = query_belief_price(&deps.querier, &pair, vault.get_swap_denom())?;
    let performance_fee = get_performance_fee(&vault, current_price)?;
    let amount_to_disburse = subtract(&vault.escrowed_amount, &performance_fee)?;

    let vault = Vault {
        escrowed_amount: empty_of(vault.escrowed_amount),
        ..vault
    };

    update_vault(deps.storage, &vault)?;
>>>>>>> 1586f6e (add updated create handlers)

    create_event(
        deps.storage,
        EventBuilder::new(
            vault.id,
            env.block.clone(),
            EventData::DcaVaultEscrowDisbursed {
                amount_disbursed: amount_to_disburse.clone(),
                performance_fee: performance_fee.clone(),
            },
        ),
    )?;

    delete_disburse_escrow_task(deps.storage, vault.id)?;

    Ok(Response::new()
        .add_submessages(get_disbursement_messages(
            deps.as_ref(),
            &vault,
            amount_to_disburse.amount,
        )?)
        .add_submessages(get_fee_messages(
            deps.as_ref(),
            env,
            vec![performance_fee.amount],
            vault.get_receive_denom(),
            true,
        )?)
        .add_attribute("performance_fee", format!("{:?}", performance_fee))
        .add_attribute("escrow_disbursed", format!("{:?}", amount_to_disburse)))
}
