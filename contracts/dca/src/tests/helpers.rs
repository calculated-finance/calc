use super::mocks::{ADMIN, DENOM_UKUJI, DENOM_UUSK, USER, VALIDATOR};
use crate::{
    constants::{ONE, TEN},
    contract::instantiate,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{
        cache::VAULT_ID_CACHE,
        pairs::save_pair,
        triggers::save_trigger,
        vaults::{get_vault, update_vault},
    },
    types::{
        config::Config,
        destination::Destination,
        event::{EventBuilder, EventData},
        fee_collector::FeeCollector,
        pair::Pair,
        performance_assessment_strategy::PerformanceAssessmentStrategy,
        position_type::PositionType,
        swap_adjustment_strategy::{
            BaseDenom, SwapAdjustmentStrategy, SwapAdjustmentStrategyParams,
        },
        time_interval::TimeInterval,
        trigger::{Trigger, TriggerConfiguration},
        vault::{Vault, VaultStatus},
    },
};
use cosmwasm_std::{
    to_binary, Addr, BlockInfo, Coin, Decimal, DepsMut, Env, MessageInfo, Timestamp, Uint128,
};
use std::{cmp::max, str::FromStr};

pub fn instantiate_contract(deps: DepsMut, env: Env, info: MessageInfo) {
    let instantiate_message = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        executors: vec![Addr::unchecked("executor")],
        fee_collectors: vec![FeeCollector {
            address: ADMIN.to_string(),
            allocation: Decimal::from_str("1").unwrap(),
        }],
        default_swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
        weighted_scale_swap_fee_percent: Decimal::percent(1),
        automation_fee_percent: Decimal::zero(),
        default_page_limit: 30,
        paused: false,
        risk_weighted_average_escrow_level: Decimal::percent(5),
        twap_period: 30,
        default_slippage_tolerance: Decimal::percent(2),
        old_staking_router_address: Addr::unchecked("staking-router"),
    };

    instantiate(deps, env.clone(), info.clone(), instantiate_message).unwrap();
}

pub fn instantiate_contract_with_multiple_fee_collectors(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fee_collectors: Vec<FeeCollector>,
) {
    let instantiate_message = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        executors: vec![Addr::unchecked("executor")],
        fee_collectors,
        default_swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
        weighted_scale_swap_fee_percent: Decimal::percent(1),
        automation_fee_percent: Decimal::from_str("0.0075").unwrap(),
        default_page_limit: 30,
        paused: false,
        risk_weighted_average_escrow_level: Decimal::from_str("0.0075").unwrap(),
        twap_period: 30,
        default_slippage_tolerance: Decimal::percent(2),
        old_staking_router_address: Addr::unchecked("staking-router"),
    };

    instantiate(deps, env.clone(), info.clone(), instantiate_message).unwrap();
}

impl Default for Config {
    fn default() -> Self {
        Self {
            admin: Addr::unchecked(ADMIN),
            executors: vec![Addr::unchecked("executor")],
            fee_collectors: vec![FeeCollector {
                address: ADMIN.to_string(),
                allocation: Decimal::from_str("1").unwrap(),
            }],
            default_swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
            weighted_scale_swap_fee_percent: Decimal::percent(1),
            automation_fee_percent: Decimal::zero(),
            default_page_limit: 30,
            paused: false,
            risk_weighted_average_escrow_level: Decimal::from_str("0.0075").unwrap(),
            twap_period: 30,
            default_slippage_tolerance: Decimal::percent(2),
            old_staking_router_address: Addr::unchecked("staking-router"),
        }
    }
}

impl Default for Destination {
    fn default() -> Self {
        Self {
            allocation: Decimal::percent(100),
            address: Addr::unchecked(USER),
            msg: None,
        }
    }
}

impl Default for Pair {
    fn default() -> Self {
        Self {
            base_denom: DENOM_UKUJI.to_string(),
            quote_denom: DENOM_UUSK.to_string(),
            address: Addr::unchecked("pair"),
        }
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            id: Uint128::zero(),
            created_at: Timestamp::default(),
            owner: Addr::unchecked(USER),
            label: Some("vault".to_string()),
            destinations: vec![Destination {
                allocation: Decimal::percent(100),
                address: Addr::unchecked("staking-router"),
                msg: Some(
                    to_binary(&ExecuteMsg::ZDelegate {
                        delegator_address: Addr::unchecked(USER),
                        validator_address: Addr::unchecked(VALIDATOR),
                    })
                    .unwrap(),
                ),
            }],
            status: VaultStatus::Active,
            balance: Coin::new(TEN.into(), DENOM_UKUJI),
            target_denom: DENOM_UUSK.to_string(),
            swap_amount: ONE,
            slippage_tolerance: Decimal::percent(10),
            minimum_receive_amount: None,
            time_interval: TimeInterval::Daily,
            started_at: None,
            escrow_level: Decimal::percent(0),
            deposited_amount: Coin::new(TEN.into(), DENOM_UKUJI),
            swapped_amount: Coin::new(0, DENOM_UKUJI),
            received_amount: Coin::new(0, DENOM_UUSK),
            escrowed_amount: Coin::new(0, DENOM_UUSK),
            trigger: Some(TriggerConfiguration::Time {
                target_time: Timestamp::from_seconds(0),
            }),
            swap_adjustment_strategy: None,
            performance_assessment_strategy: None,
        }
    }
}

impl Default for SwapAdjustmentStrategy {
    fn default() -> Self {
        Self::RiskWeightedAverage {
            model_id: 30,
            base_denom: BaseDenom::Bitcoin,
            position_type: PositionType::Enter,
        }
    }
}

impl Default for SwapAdjustmentStrategyParams {
    fn default() -> Self {
        Self::RiskWeightedAverage {
            base_denom: BaseDenom::Bitcoin,
        }
    }
}

impl Default for PerformanceAssessmentStrategy {
    fn default() -> Self {
        Self::CompareToStandardDca {
            swapped_amount: Coin::new(0, DENOM_UKUJI),
            received_amount: Coin::new(0, DENOM_UUSK),
        }
    }
}

impl Default for EventBuilder {
    fn default() -> Self {
        EventBuilder::new(
            Uint128::one(),
            BlockInfo {
                height: 23498723,
                time: Timestamp::from_seconds(1681711929),
                chain_id: "test".to_string(),
            },
            EventData::default(),
        )
    }
}

impl Default for EventData {
    fn default() -> Self {
        EventData::DcaVaultExecutionTriggered {
            base_denom: DENOM_UUSK.to_string(),
            quote_denom: DENOM_UKUJI.to_string(),
            asset_price: Decimal::new(Uint128::one()),
        }
    }
}

pub fn setup_vault(deps: DepsMut, env: Env, mut vault: Vault) -> Vault {
    save_pair(
        deps.storage,
        &Pair {
            quote_denom: vault.balance.denom.clone(),
            base_denom: vault.target_denom.clone(),
            ..Pair::default()
        },
    )
    .unwrap();

    let mut existing_vault = get_vault(deps.storage, vault.id);

    while existing_vault.is_ok() {
        vault.id = existing_vault.unwrap().id + Uint128::one();
        existing_vault = get_vault(deps.storage, vault.id);
    }

    update_vault(deps.storage, vault.clone()).unwrap();

    if vault.trigger.is_some() {
        save_trigger(
            deps.storage,
            Trigger {
                vault_id: vault.id,
                configuration: match vault.trigger.unwrap() {
                    TriggerConfiguration::Time { target_time } => TriggerConfiguration::Time {
                        target_time: max(target_time, env.block.time),
                    },
                    TriggerConfiguration::Price {
                        target_price,
                        order_idx,
                    } => TriggerConfiguration::Price {
                        target_price,
                        order_idx,
                    },
                },
            },
        )
        .unwrap();
    }

    VAULT_ID_CACHE.save(deps.storage, &vault.id).unwrap();

    get_vault(deps.storage, vault.id).unwrap()
}
