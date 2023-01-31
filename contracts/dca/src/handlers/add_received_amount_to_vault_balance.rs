use crate::{
    error::ContractError,
    state::{
        cache::{CACHE, SWAP_CACHE},
        vaults::{get_vault, update_vault},
    },
};
use cosmwasm_std::{Coin, DepsMut, Env, Response, StdError};

pub fn add_received_amount_to_vault_balance(
    deps: DepsMut,
    env: Env,
) -> Result<Response, ContractError> {
    let swap_cache = SWAP_CACHE.load(deps.storage)?;

    let vault_swap_denom_balance = &deps.querier.query_balance(
        env.contract.address,
        &swap_cache.receive_denom_balance.denom,
    )?;

    let balance_received = Coin::new(
        (vault_swap_denom_balance
            .amount
            .checked_sub(swap_cache.receive_denom_balance.amount)
            .expect(&format!(
                "Amount of {} to add to vault balance",
                vault_swap_denom_balance.denom
            )))
        .into(),
        vault_swap_denom_balance.denom.clone(),
    );

    let vault = get_vault(deps.storage, CACHE.load(deps.storage)?.vault_id.into())?;

    update_vault(deps.storage, vault.id, |stored_vault| {
        if let Some(mut stored_vault) = stored_vault {
            if stored_vault.balance.denom == balance_received.denom {
                stored_vault.balance.amount += balance_received.amount;
            } else {
                stored_vault.balance = balance_received;
            }
            return Ok(stored_vault);
        }
        Err(StdError::GenericErr {
            msg: format!("Vault {}", vault.id),
        })
    })?;

    Ok(Response::new())
}
