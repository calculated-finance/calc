use cosmwasm_std::{Order, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Map, PrefixBound};
use std::marker::PhantomData;

pub const CLAIM_ESCROW_TASKS_BY_TIMESTAMP: Map<(u64, u128), u128> =
    Map::new("claim_escrow_task_by_timestamp_v20");

pub fn save_claim_escrow_task(
    store: &mut dyn Storage,
    vault_id: Uint128,
    timestamp: Timestamp,
) -> StdResult<()> {
    CLAIM_ESCROW_TASKS_BY_TIMESTAMP.save(
        store,
        (timestamp.seconds(), vault_id.into()),
        &vault_id.into(),
    )
}

pub fn get_claim_escrow_tasks(
    store: &dyn Storage,
    due_before: Timestamp,
) -> StdResult<Vec<Uint128>> {
    Ok(CLAIM_ESCROW_TASKS_BY_TIMESTAMP
        .prefix_range(
            store,
            None,
            Some(PrefixBound::Inclusive((due_before.seconds(), PhantomData))),
            Order::Ascending,
        )
        .flat_map(|result| result.map(|(_, vault_id)| vault_id.into()))
        .collect::<Vec<Uint128>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::Uint128;

    #[test]
    fn fetches_vault_ids_for_tasks_that_are_due() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let vault_id = Uint128::one();

        save_claim_escrow_task(&mut deps.storage, vault_id, env.block.time).unwrap();

        let vault_ids =
            get_claim_escrow_tasks(&deps.storage, env.block.time.plus_seconds(10)).unwrap();

        assert_eq!(vault_ids, vec![vault_id]);
    }

    #[test]
    fn does_not_fetch_vault_ids_for_tasks_that_are_not_due() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        save_claim_escrow_task(
            &mut deps.storage,
            Uint128::one(),
            env.block.time.plus_seconds(10),
        )
        .unwrap();

        let vault_ids = get_claim_escrow_tasks(&deps.storage, env.block.time).unwrap();

        assert!(vault_ids.is_empty());
    }

    #[test]
    fn stores_and_fetches_separate_tasks_at_the_same_timestamp() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let vault_id_1 = Uint128::one();
        let vault_id_2 = Uint128::new(2);

        save_claim_escrow_task(&mut deps.storage, vault_id_1, env.block.time).unwrap();
        save_claim_escrow_task(&mut deps.storage, vault_id_2, env.block.time).unwrap();

        let vault_ids =
            get_claim_escrow_tasks(&deps.storage, env.block.time.plus_seconds(10)).unwrap();

        assert_eq!(vault_ids, vec![vault_id_1, vault_id_2]);
    }
}
