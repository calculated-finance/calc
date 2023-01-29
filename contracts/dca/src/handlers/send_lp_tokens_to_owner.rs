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

#[cfg(test)]
mod send_lp_tokens_to_owner_tests {
    use super::send_lp_tokens_to_owner;
    use crate::{
        contract::AFTER_SENDING_LP_TOKENS_TO_OWNER,
        state::{
            cache::{BowCache, BOW_CACHE},
            vaults::get_vault,
        },
        tests::{
            helpers::{instantiate_contract, setup_vault},
            mocks::ADMIN,
        },
        types::source::Source,
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, BankMsg, Coin, CosmosMsg, SubMsg, Uint128,
    };

    #[test]
    pub fn sends_lp_tokens_to_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        let pool_address = Addr::unchecked("bow-pool");

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Coin::new(1000000, "ukuji"),
            Uint128::new(10000),
            Some(Source::Bow {
                address: pool_address.clone(),
            }),
        );

        let lp_token_balance = Coin::new(10000, format!("factory/{}/ulp", pool_address));

        BOW_CACHE
            .save(
                deps.as_mut().storage,
                &BowCache {
                    pool_address: pool_address.clone(),
                    lp_token_balance: Some(lp_token_balance.clone()),
                    withdrawal: vec![],
                    deposit: vec![],
                },
            )
            .unwrap();

        deps.querier
            .update_balance("cosmos2contract", vec![lp_token_balance.clone()]);

        let response = send_lp_tokens_to_owner(deps.as_mut(), env.clone()).unwrap();

        assert!(response.messages.contains(&SubMsg::reply_on_success(
            CosmosMsg::Bank(BankMsg::Send {
                to_address: vault.owner.to_string(),
                amount: vec![lp_token_balance],
            }),
            AFTER_SENDING_LP_TOKENS_TO_OWNER,
        )));
    }

    #[test]
    pub fn updates_the_vault_balance_to_lp_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        let pool_address = Addr::unchecked("bow-pool");

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Coin::new(1000000, "ukuji"),
            Uint128::new(10000),
            Some(Source::Bow {
                address: pool_address.clone(),
            }),
        );

        let lp_token_balance = Coin::new(10000, format!("factory/{}/ulp", pool_address));

        BOW_CACHE
            .save(
                deps.as_mut().storage,
                &BowCache {
                    pool_address: pool_address.clone(),
                    lp_token_balance: Some(lp_token_balance.clone()),
                    withdrawal: vec![],
                    deposit: vec![],
                },
            )
            .unwrap();

        deps.querier
            .update_balance("cosmos2contract", vec![lp_token_balance.clone()]);

        send_lp_tokens_to_owner(deps.as_mut(), env.clone()).unwrap();

        let updated_vault = get_vault(deps.as_mut().storage, vault.id).unwrap();

        assert_eq!(updated_vault.balance, lp_token_balance);
    }

    #[test]
    pub fn adds_lp_tokens_to_existing_vault_balance() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        let pool_address = Addr::unchecked("bow-pool");

        instantiate_contract(deps.as_mut(), env.clone(), info.clone());

        let vault = setup_vault(
            deps.as_mut(),
            env.clone(),
            Coin::new(10000, format!("factory/{}/ulp", pool_address)),
            Uint128::new(10000),
            Some(Source::Bow {
                address: pool_address.clone(),
            }),
        );

        let lp_token_balance = Coin::new(10000, format!("factory/{}/ulp", pool_address));

        BOW_CACHE
            .save(
                deps.as_mut().storage,
                &BowCache {
                    pool_address: pool_address.clone(),
                    lp_token_balance: Some(lp_token_balance.clone()),
                    withdrawal: vec![],
                    deposit: vec![],
                },
            )
            .unwrap();

        deps.querier
            .update_balance("cosmos2contract", vec![lp_token_balance.clone()]);

        send_lp_tokens_to_owner(deps.as_mut(), env.clone()).unwrap();

        let updated_vault = get_vault(deps.as_mut().storage, vault.id).unwrap();

        assert_eq!(
            updated_vault.balance.amount,
            vault.balance.amount + lp_token_balance.amount
        );
        assert_eq!(updated_vault.balance.denom, lp_token_balance.denom);
    }
}
