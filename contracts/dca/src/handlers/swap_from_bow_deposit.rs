use crate::{
    contract::AFTER_SWAPPING_FROM_BOW_DEPOSIT,
    error::ContractError,
    msg::ExecuteMsg,
    state::{
        cache::{BOW_CACHE, CACHE},
        pairs::find_pair,
        vaults::{get_vault, update_vault},
    },
    types::reply_config::ReplyConfig,
};
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, DepsMut, Empty, Env, ReplyOn, Response, StdError, SubMsg, WasmMsg,
};

pub fn swap_from_bow_deposit(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let bow_cache = BOW_CACHE.load(deps.storage)?;
    let vault = get_vault(deps.storage, CACHE.load(deps.storage)?.vault_id.into())?;
    let swap_denom = vault.get_swap_denom();

    let swap_messages = bow_cache
        .withdrawal
        .iter()
        .map(|old_balance| {
            let new_balance = deps
                .querier
                .query_balance(env.contract.address.to_string(), old_balance.denom.as_str())
                .expect(format!("Balance for {}", old_balance.denom).as_str());

            let amount_returned_from_bow = Coin::new(
                new_balance
                    .amount
                    .checked_sub(old_balance.amount.clone())
                    .expect(&format!(
                        "Amount of {} returned from BOW",
                        old_balance.denom.clone()
                    ))
                    .into(),
                old_balance.denom.clone(),
            );

            let denoms = [old_balance.denom.clone(), swap_denom.to_string()];

            if denoms[0] == denoms[1] {
                update_vault(deps.storage, vault.id, |stored_vault| match stored_vault {
                    Some(mut stored_vault) => {
                        stored_vault.balance = amount_returned_from_bow;
                        Ok(stored_vault)
                    }
                    None => Err(StdError::generic_err(format!(
                        "Vault {} not exist",
                        vault.id
                    ))),
                })
                .expect("Updated vault");

                return None;
            }

            let pair = find_pair(deps.storage, denoms.clone())
                .expect(&format!("Pair for denoms {:?}", denoms));

            Some(SubMsg::new(CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Swap {
                    pair_address: pair.address,
                    slippage_tolerance: vault.slippage_tolerance,
                    reply_config: Some(ReplyConfig {
                        id: AFTER_SWAPPING_FROM_BOW_DEPOSIT,
                        on: ReplyOn::Success,
                    }),
                })
                .expect("swap on fin message"),
                funds: vec![amount_returned_from_bow],
            })))
        })
        .flatten()
        .collect::<Vec<SubMsg>>();

    Ok(Response::new().add_submessages(swap_messages))
}
