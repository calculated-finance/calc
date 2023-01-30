use crate::{
    handlers::add_received_amount_to_vault_balance::add_received_amount_to_vault_balance,
    state::{
        cache::{SwapCache, SWAP_CACHE},
        vaults::{get_vault, update_vault},
    },
    tests::helpers::setup_active_vault_with_funds,
};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env},
    Coin, StdError,
};

#[test]
pub fn adds_received_amount_to_existing_vault_balance() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());
    let balance_received = Coin::new(1000000, vault.get_swap_denom());

    SWAP_CACHE
        .save(
            deps.as_mut().storage,
            &SwapCache {
                swap_denom_balance: vault.balance.clone(),
                receive_denom_balance: Coin::new(0, vault.get_receive_denom()),
            },
        )
        .unwrap();

    deps.querier.update_balance(
        env.contract.address.clone(),
        vec![Coin::new(
            (vault.balance.amount + balance_received.amount).into(),
            vault.get_swap_denom(),
        )],
    );

    add_received_amount_to_vault_balance(deps.as_mut(), env).unwrap();
    let updated_vault = get_vault(&deps.storage, vault.id).unwrap();

    assert_eq!(
        updated_vault.balance.amount,
        vault.balance.amount + balance_received.amount
    );
}

#[test]
pub fn replaces_different_denom_vault_balance_with_received_amount() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let vault = setup_active_vault_with_funds(deps.as_mut(), env.clone());

    update_vault(deps.as_mut().storage, vault.id, |stored_vault| {
        if let Some(mut stored_vault) = stored_vault {
            stored_vault.balance.denom = "different_denom".to_string();
            return Ok(stored_vault);
        }
        Err(StdError::GenericErr {
            msg: format!("Vault {}", vault.id),
        })
    })
    .unwrap();

    let balance_received = Coin::new(1000000, vault.get_swap_denom());

    SWAP_CACHE
        .save(
            deps.as_mut().storage,
            &SwapCache {
                swap_denom_balance: Coin::new(0, vault.get_swap_denom()),
                receive_denom_balance: Coin::new(0, vault.get_receive_denom()),
            },
        )
        .unwrap();

    deps.querier.update_balance(
        env.contract.address.clone(),
        vec![Coin::new(
            (balance_received.amount).into(),
            vault.get_swap_denom(),
        )],
    );

    add_received_amount_to_vault_balance(deps.as_mut(), env).unwrap();
    let updated_vault = get_vault(&deps.storage, vault.id).unwrap();

    assert_eq!(updated_vault.balance.amount, balance_received.amount);
}
