use crate::{
    contract::AFTER_SENDING_LP_TOKENS_TO_OWNER,
    error::ContractError,
    state::{
        cache::{BOW_CACHE, CACHE},
        vaults::{get_vault, update_vault},
    },
    types::vault::Vault,
};
use cosmwasm_std::{BankMsg, CosmosMsg, DepsMut, Env, Response, StdError, StdResult, SubMsg};

pub fn send_lp_tokens_to_owner(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let vault = get_vault(deps.storage, CACHE.load(deps.storage)?.vault_id.into())?;
    let mut cache = BOW_CACHE.load(deps.storage)?;

    let lp_token_balance = deps.querier.query_balance(
        &env.contract.address,
        format!("factory/{}/ulp", cache.pool_address),
    )?;

    cache.lp_token_balance = Some(lp_token_balance.clone());
    BOW_CACHE.save(deps.storage, &cache)?;

    update_vault(deps.storage, vault.id, |stored_vault| -> StdResult<Vault> {
        match stored_vault {
            Some(mut stored_vault) => {
                if stored_vault.balance.denom == lp_token_balance.denom {
                    stored_vault.balance.amount += lp_token_balance.amount;
                } else {
                    stored_vault.balance = lp_token_balance.clone();
                }
                Ok(stored_vault)
            }
            None => Err(StdError::NotFound {
                kind: format!("vault for address: {} with id: {}", vault.owner, vault.id),
            }),
        }
    })?;

    Ok(Response::new()
        .add_attribute("lp_tokens_minted", format!("{:?}", lp_token_balance))
        .add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Bank(BankMsg::Send {
                to_address: vault.owner.to_string(),
                amount: vec![lp_token_balance],
            }),
            AFTER_SENDING_LP_TOKENS_TO_OWNER,
        )))
}
