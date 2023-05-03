use super::{
    dca_plus_config::{dca_plus_config_from, get_dca_plus_config, DCA_PLUS_CONFIGS},
    destinations::{destination_from, get_destinations, OldDestination, DESTINATIONS},
    pairs::{find_pair, find_pair_by_address},
    triggers::get_trigger,
};
use crate::{
    helpers::state::fetch_and_increment_counter,
    types::{
        dca_plus_config::DcaPlusConfig,
        pair::Pair,
        performance_assessment_strategy::PerformanceAssessmentStrategy,
        price_delta_limit::PriceDeltaLimit,
        swap_adjustment_strategy::{BaseDenom, SwapAdjustmentStrategy},
        time_interval::TimeInterval,
        trigger::TriggerConfiguration,
        vault::{Vault, VaultBuilder, VaultStatus},
    },
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Env, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, Item, UniqueIndex};

const VAULT_COUNTER: Item<u64> = Item::new("vault_counter_v20");

#[cw_serde]
struct OldVault {
    pub id: Uint128,
    pub created_at: Timestamp,
    pub owner: Addr,
    pub label: Option<String>,
    pub destinations: Vec<OldDestination>,
    pub status: VaultStatus,
    pub balance: Coin,
    pub pair_address: Addr,
    pub swap_amount: Uint128,
    pub slippage_tolerance: Option<Decimal>,
    pub minimum_receive_amount: Option<Uint128>,
    pub time_interval: TimeInterval,
    pub started_at: Option<Timestamp>,
    pub swapped_amount: Coin,
    pub received_amount: Coin,
    pub price_delta_limits: Vec<PriceDeltaLimit>,
}

fn old_vault_from(storage: &dyn Storage, vault: &Vault) -> StdResult<OldVault> {
    let pair = find_pair(storage, &vault.denoms())?;
    Ok(OldVault {
        id: vault.id,
        created_at: vault.created_at,
        owner: vault.owner.clone(),
        label: vault.label.clone(),
        destinations: vec![],
        status: vault.status.clone(),
        balance: vault.balance.clone(),
        pair_address: pair.address.clone(),
        swap_amount: vault.swap_amount,
        slippage_tolerance: vault.slippage_tolerance,
        minimum_receive_amount: vault.minimum_receive_amount,
        time_interval: vault.time_interval.clone(),
        started_at: vault.started_at,
        swapped_amount: vault.swapped_amount.clone(),
        received_amount: vault.received_amount.clone(),
        price_delta_limits: vec![],
    })
}

fn vault_from(
    env: &Env,
    data: &OldVault,
    pair: Pair,
    trigger: Option<TriggerConfiguration>,
    destinations: &mut Vec<OldDestination>,
    dca_plus_config: Option<DcaPlusConfig>,
) -> Vault {
    destinations.append(
        &mut data
            .destinations
            .clone()
            .into_iter()
            .map(|destination| destination.into())
            .collect(),
    );
    Vault {
        id: data.id,
        created_at: data.created_at,
        owner: data.owner.clone(),
        label: data.label.clone(),
        destinations: destinations
            .into_iter()
            .map(|d| destination_from(&d, data.owner.clone(), env.contract.address.clone()))
            .collect(),
        status: data.status.clone(),
        balance: data.balance.clone(),
        target_denom: pair.other_denom(data.balance.denom.clone()),
        swap_amount: data.swap_amount,
        slippage_tolerance: data.slippage_tolerance,
        minimum_receive_amount: data.minimum_receive_amount,
        time_interval: data.time_interval.clone(),
        started_at: data.started_at,
        escrow_level: dca_plus_config
            .clone()
            .map_or(Decimal::zero(), |dca_plus_config| {
                dca_plus_config.escrow_level
            }),
        escrowed_amount: dca_plus_config.clone().map_or(
            Coin::new(0, data.balance.denom.clone()),
            |dca_plus_config| dca_plus_config.escrowed_balance,
        ),
        swapped_amount: data.swapped_amount.clone(),
        deposited_amount: dca_plus_config
            .clone()
            .map_or(data.balance.clone(), |dca_plus_config| {
                dca_plus_config.total_deposit
            }),
        received_amount: data.received_amount.clone(),
        trigger,
        swap_adjustment_strategy: dca_plus_config.clone().map(|dca_plus_config| {
            SwapAdjustmentStrategy::RiskWeightedAverage {
                model_id: dca_plus_config.model_id,
                base_denom: BaseDenom::Bitcoin,
                position_type: pair.position_type(data.balance.denom.clone()),
            }
        }),
        performance_assessment_strategy: dca_plus_config.map(|dca_plus_config| {
            PerformanceAssessmentStrategy::CompareToStandardDca {
                swapped_amount: dca_plus_config.standard_dca_swapped_amount,
                received_amount: dca_plus_config.standard_dca_received_amount,
            }
        }),
    }
}

struct VaultIndexes<'a> {
    pub owner: UniqueIndex<'a, (Addr, u128), OldVault, u128>,
    pub owner_status: UniqueIndex<'a, (Addr, u8, u128), OldVault, u128>,
}

impl<'a> IndexList<OldVault> for VaultIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<OldVault>> + '_> {
        let v: Vec<&dyn Index<OldVault>> = vec![&self.owner, &self.owner_status];
        Box::new(v.into_iter())
    }
}

fn vault_store<'a>() -> IndexedMap<'a, u128, OldVault, VaultIndexes<'a>> {
    let indexes = VaultIndexes {
        owner: UniqueIndex::new(|v| (v.owner.clone(), v.id.into()), "vaults_v20__owner"),
        owner_status: UniqueIndex::new(
            |v| (v.owner.clone(), v.status.clone() as u8, v.id.into()),
            "vaults_v20__owner_status",
        ),
    };
    IndexedMap::new("vaults_v20", indexes)
}

pub fn save_vault(store: &mut dyn Storage, vault_builder: VaultBuilder) -> StdResult<Vault> {
    let vault = vault_builder.build(fetch_and_increment_counter(store, VAULT_COUNTER)?.into());
    DESTINATIONS.save(
        store,
        vault.id.into(),
        &to_binary(&vault.destinations).expect("serialised destinations"),
    )?;
    if let Some(dca_plus_config) = dca_plus_config_from(&vault) {
        DCA_PLUS_CONFIGS.save(store, vault.id.into(), &dca_plus_config)?;
    }
    let old_vault = &old_vault_from(store, &vault)?;
    vault_store().save(store, vault.id.into(), old_vault)?;
    Ok(vault)
}

pub fn get_vault(env: &Env, store: &dyn Storage, vault_id: Uint128) -> StdResult<Vault> {
    let data = vault_store().load(store, vault_id.into())?;
    Ok(vault_from(
        env,
        &data,
        find_pair_by_address(store, data.pair_address.clone())?,
        get_trigger(store, vault_id)?.map(|t| t.configuration),
        &mut get_destinations(store, vault_id)?,
        get_dca_plus_config(store, vault_id),
    ))
}

pub fn get_vaults_by_address(
    store: &dyn Storage,
    env: &Env,
    address: Addr,
    status: Option<VaultStatus>,
    start_after: Option<Uint128>,
    limit: Option<u16>,
) -> StdResult<Vec<Vault>> {
    let partition = match status {
        Some(status) => vault_store()
            .idx
            .owner_status
            .prefix((address, status as u8)),
        None => vault_store().idx.owner.prefix(address),
    };

    Ok(partition
        .range(
            store,
            start_after.map(|vault_id| Bound::exclusive(vault_id)),
            None,
            cosmwasm_std::Order::Ascending,
        )
        .take(limit.unwrap_or(30) as usize)
        .map(|result| {
            let (_, vault_data) =
                result.expect(format!("a vault with id after {:?}", start_after).as_str());
            vault_from(
                env,
                &vault_data,
                find_pair_by_address(store, vault_data.pair_address.clone()).expect(
                    format!("a pair for pair address {:?}", vault_data.pair_address).as_str(),
                ),
                get_trigger(store, vault_data.id.into())
                    .expect(format!("a trigger for vault id {}", vault_data.id).as_str())
                    .map(|trigger| trigger.configuration),
                &mut get_destinations(store, vault_data.id).expect("vault destinations"),
                get_dca_plus_config(store, vault_data.id),
            )
        })
        .collect::<Vec<Vault>>())
}

pub fn get_vaults(
    env: &Env,
    store: &dyn Storage,
    start_after: Option<Uint128>,
    limit: Option<u16>,
) -> StdResult<Vec<Vault>> {
    Ok(vault_store()
        .range(
            store,
            start_after.map(|vault_id| Bound::exclusive(vault_id)),
            None,
            cosmwasm_std::Order::Ascending,
        )
        .take(limit.unwrap_or(30) as usize)
        .map(|result| {
            let (_, vault_data) =
                result.expect(format!("a vault with id after {:?}", start_after).as_str());
            vault_from(
                env,
                &vault_data,
                find_pair_by_address(store, vault_data.pair_address.clone()).expect(
                    format!("a pair for pair address {:?}", vault_data.pair_address).as_str(),
                ),
                get_trigger(store, vault_data.id.into())
                    .expect(format!("a trigger for vault id {}", vault_data.id).as_str())
                    .map(|trigger| trigger.configuration),
                &mut get_destinations(store, vault_data.id).expect("vault destinations"),
                get_dca_plus_config(store, vault_data.id),
            )
        })
        .collect::<Vec<Vault>>())
}

pub fn update_vault(store: &mut dyn Storage, vault: &Vault) -> StdResult<()> {
    DESTINATIONS.save(
        store,
        vault.id.into(),
        &to_binary(&vault.destinations).expect("serialised destinations"),
    )?;
    if let Some(dca_plus_config) = dca_plus_config_from(vault) {
        DCA_PLUS_CONFIGS.save(store, vault.id.into(), &dca_plus_config)?;
    }
    let old_vault = &old_vault_from(store, vault)?;
    vault_store().save(store, vault.id.into(), old_vault)
}

pub fn clear_vaults(store: &mut dyn Storage) {
    vault_store().clear(store);
    VAULT_COUNTER.remove(store)
}

#[cfg(test)]
mod destination_store_tests {
    use super::*;
    use crate::{
        state::pairs::save_pair,
        types::{destination::Destination, vault::VaultBuilder},
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, Coin, Decimal, Env, Uint128,
    };

    fn create_vault_builder(env: Env) -> VaultBuilder {
        VaultBuilder::new(
            env.block.time,
            Addr::unchecked("owner"),
            None,
            vec![Destination {
                address: Addr::unchecked("owner"),
                allocation: Decimal::one(),
                msg: None,
            }],
            VaultStatus::Active,
            Coin::new(1000u128, "ukuji".to_string()),
            "demo".to_string(),
            Uint128::new(100),
            None,
            None,
            None,
            TimeInterval::Daily,
            None,
            Decimal::zero(),
            Coin::new(0, "ukuji".to_string()),
            Coin::new(0, "demo".to_string()),
            Coin::new(0, "demo".to_string()),
            None,
            None,
        )
    }

    #[test]
    fn destinations_are_returned() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let store = deps.as_mut().storage;

        let pair = Pair::default();

        save_pair(store, &pair.clone()).unwrap();

        let vault_builder = create_vault_builder(env.clone());
        let mut vault = save_vault(store, vault_builder).unwrap();

        vault.status = VaultStatus::Inactive;
        update_vault(store, &vault).unwrap();

        let fetched_vault = get_vault(&env, store, vault.id).unwrap();
        assert_eq!(fetched_vault.destinations, vault.destinations);
        assert!(!fetched_vault.destinations.is_empty());
    }
}
