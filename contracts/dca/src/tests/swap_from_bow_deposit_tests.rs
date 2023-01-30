use crate::{
    contract::AFTER_SWAPPING_FROM_BOW_DEPOSIT,
    handlers::swap_from_bow_deposit::swap_from_bow_deposit,
    msg::ExecuteMsg,
    state::{
        cache::{BowCache, BOW_CACHE},
        vaults::{get_vault, update_vault},
    },
    tests::helpers::setup_vault,
    types::{reply_config::ReplyConfig, source::Source},
};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    to_binary, Addr, Coin, CosmosMsg, ReplyOn, StdError, SubMsg, Uint128, WasmMsg,
};

#[test]
fn adds_swap_denom_to_vault_balance() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let pool_address = Addr::unchecked("bow-pool");

    let vault = setup_vault(
        deps.as_mut(),
        env.clone(),
        Coin::new(10000000, "base"),
        Uint128::new(100000),
        Some(Source::Bow {
            address: pool_address.clone(),
        }),
    );

    update_vault(deps.as_mut().storage, vault.id, |stored_vault| {
        if let Some(mut stored_vault) = stored_vault {
            stored_vault.balance =
                Coin::new(10000000, format!("factory/{}/ulp", pool_address.clone()));
            return Ok(stored_vault);
        }
        Err(StdError::generic_err(format!(
            "Vault {} not exist",
            vault.id
        )))
    })
    .unwrap();

    BOW_CACHE
        .save(
            deps.as_mut().storage,
            &BowCache {
                pool_address,
                withdrawal: vec![Coin::new(0, vault.get_swap_denom()), Coin::new(0, "quote")],
                deposit: vec![],
                lp_token_balance: None,
            },
        )
        .unwrap();

    let withdrawn_swap_denom = Coin::new(100000, vault.get_swap_denom());

    deps.querier.update_balance(
        env.contract.address.clone(),
        vec![withdrawn_swap_denom.clone(), Coin::new(10000, "quote")],
    );

    swap_from_bow_deposit(deps.as_mut(), env).unwrap();

    let updated_vault = get_vault(deps.as_ref().storage, vault.id).unwrap();

    assert_eq!(updated_vault.balance, withdrawn_swap_denom);
}

#[test]
fn sends_swap_message_for_non_swap_denom() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let pool_address = Addr::unchecked("bow-pool");

    let vault = setup_vault(
        deps.as_mut(),
        env.clone(),
        Coin::new(10000000, "base"),
        Uint128::new(100000),
        Some(Source::Bow {
            address: pool_address.clone(),
        }),
    );

    update_vault(deps.as_mut().storage, vault.id, |stored_vault| {
        if let Some(mut stored_vault) = stored_vault {
            stored_vault.balance =
                Coin::new(10000000, format!("factory/{}/ulp", pool_address.clone()));
            return Ok(stored_vault);
        }
        Err(StdError::generic_err(format!(
            "Vault {} not exist",
            vault.id
        )))
    })
    .unwrap();

    BOW_CACHE
        .save(
            deps.as_mut().storage,
            &BowCache {
                pool_address,
                withdrawal: vec![Coin::new(0, "base"), Coin::new(0, "quote")],
                deposit: vec![],
                lp_token_balance: None,
            },
        )
        .unwrap();

    let withdrawn_non_swap_denom = Coin::new(10000, "quote");

    deps.querier.update_balance(
        env.contract.address.to_string(),
        vec![Coin::new(10000, "base"), withdrawn_non_swap_denom.clone()],
    );

    let response = swap_from_bow_deposit(deps.as_mut(), env.clone()).unwrap();

    assert!(response
        .messages
        .contains(&SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Swap {
                pair_address: vault.pair.address,
                slippage_tolerance: vault.slippage_tolerance,
                reply_config: Some(ReplyConfig {
                    id: AFTER_SWAPPING_FROM_BOW_DEPOSIT,
                    on: ReplyOn::Success,
                }),
            })
            .expect("swap on fin message"),
            funds: vec![withdrawn_non_swap_denom],
        }))))
}
