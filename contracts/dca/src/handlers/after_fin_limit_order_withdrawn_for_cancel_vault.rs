use crate::{
    error::ContractError,
    state::{delete_trigger, get_vault, update_vault, CACHE, LIMIT_ORDER_CACHE},
    vault::Vault,
};
use base::vaults::vault::VaultStatus;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{BankMsg, Coin, DepsMut, Env, Reply, Response};
use cosmwasm_std::{CosmosMsg, StdError, StdResult, Uint128};

pub fn after_fin_limit_order_withdrawn_for_cancel_vault(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, cache.vault_id.into())?;
    match reply.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            let limit_order_cache = LIMIT_ORDER_CACHE.load(deps.storage)?;

            // send assets from partially filled order to owner
            let filled_amount = Coin {
                denom: vault.get_receive_denom().clone(),
                amount: limit_order_cache.filled,
            };

            // i dont think its possible for this to be zero
            let mut filled_amount_bank_msgs: Vec<CosmosMsg> = Vec::new();
            if vault.balance.amount.gt(&Uint128::zero()) {
                filled_amount_bank_msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: vault.owner.to_string(),
                    amount: vec![filled_amount.clone()],
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
                            kind: format!(
                                "vault for address: {} with id: {}",
                                vault.owner, vault.id
                            ),
                        }),
                    }
                },
            )?;

            delete_trigger(deps.storage, vault.id.into())?;

            Ok(Response::new()
                .add_attribute("method", "fin_limit_order_withdrawn_for_cancel_vault")
                .add_messages(filled_amount_bank_msgs))
        }
        cosmwasm_std::SubMsgResult::Err(e) => Err(ContractError::CustomError {
            val: format!(
                "failed to withdraw fin limit order for vault id: {} - {}",
                vault.id, e
            ),
        }),
    }
}
