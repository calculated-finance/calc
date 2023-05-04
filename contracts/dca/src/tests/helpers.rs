use super::mocks::{ADMIN, DENOM_UDEMO, DENOM_UKUJI, USER, VALIDATOR};
use crate::{
    constants::{ONE, TEN},
    contract::instantiate,
    handlers::get_vault::get_vault_handler,
    helpers::price::FinSimulationResponse,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{
        cache::{VaultCache, VAULT_CACHE},
        pairs::save_pair,
        triggers::save_trigger,
        vaults::update_vault,
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
    from_binary,
    testing::{MockApi, MockQuerier},
    to_binary, Addr, BlockInfo, Coin, ContractResult, Decimal, DepsMut, Env, MemoryStorage,
    MessageInfo, OwnedDeps, SystemResult, Timestamp, Uint128, WasmQuery,
};
use kujira::fin::QueryMsg;
use std::{cmp::max, str::FromStr};

pub fn instantiate_contract(deps: DepsMut, env: Env, info: MessageInfo) {
    let instantiate_message = InstantiateMsg {
        admin: Addr::unchecked(ADMIN),
        executors: vec![Addr::unchecked("executor")],
        fee_collectors: vec![FeeCollector {
            address: ADMIN.to_string(),
            allocation: Decimal::from_str("1").unwrap(),
        }],
        swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
        delegation_fee_percent: Decimal::from_str("0.0075").unwrap(),
        page_limit: 1000,
        paused: false,
        risk_weighted_average_escrow_level: Decimal::percent(5),
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
        swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
        delegation_fee_percent: Decimal::from_str("0.0075").unwrap(),
        page_limit: 1000,
        paused: false,
        risk_weighted_average_escrow_level: Decimal::from_str("0.0075").unwrap(),
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
            swap_fee_percent: Decimal::from_str("0.0165").unwrap(),
            delegation_fee_percent: Decimal::from_str("0.0075").unwrap(),
            page_limit: 1000,
            paused: false,
            risk_weighted_average_escrow_level: Decimal::from_str("0.0075").unwrap(),
        }
    }
}

impl Default for Pair {
    fn default() -> Self {
        Self {
            address: Addr::unchecked("pair"),
            base_denom: DENOM_UDEMO.to_string(),
            quote_denom: DENOM_UKUJI.to_string(),
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

impl Default for Vault {
    fn default() -> Self {
        Self {
            id: Uint128::zero(),
            created_at: Timestamp::default(),
            owner: Addr::unchecked(USER),
            label: Some("vault".to_string()),
            destinations: vec![Destination {
                allocation: Decimal::percent(100),
                address: Addr::unchecked("contractaddress"),
                msg: Some(
                    to_binary(&ExecuteMsg::ZDelegate {
                        delegator_address: Addr::unchecked(USER),
                        validator_address: Addr::unchecked(VALIDATOR),
                    })
                    .unwrap(),
                ),
            }],
            status: VaultStatus::Active,
            balance: Coin::new(TEN.into(), DENOM_UDEMO),
            target_denom: DENOM_UKUJI.to_string(),
            swap_amount: ONE,
            slippage_tolerance: None,
            minimum_receive_amount: None,
            time_interval: TimeInterval::Daily,
            started_at: None,
            escrow_level: Decimal::percent(0),
            deposited_amount: Coin::new(TEN.into(), DENOM_UDEMO),
            swapped_amount: Coin::new(0, DENOM_UDEMO),
            received_amount: Coin::new(0, DENOM_UKUJI),
            escrowed_amount: Coin::new(0, DENOM_UKUJI),
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
            swapped_amount: Coin::new(0, DENOM_UDEMO),
            received_amount: Coin::new(0, DENOM_UKUJI),
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
            base_denom: DENOM_UKUJI.to_string(),
            quote_denom: DENOM_UDEMO.to_string(),
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

    let mut existing_vault = get_vault_handler(deps.as_ref(), vault.id);

    while existing_vault.is_ok() {
        vault.id = existing_vault.unwrap().vault.id + Uint128::one();
        existing_vault = get_vault_handler(deps.as_ref(), vault.id);
    }

    update_vault(deps.storage, &vault).unwrap();

    if let Some(TriggerConfiguration::Time { target_time }) = vault.trigger {
        let trigger_time =
            Timestamp::from_seconds(max(target_time.seconds(), env.block.time.seconds()));

        save_trigger(
            deps.storage,
            Trigger {
                vault_id: vault.id,
                configuration: TriggerConfiguration::Time {
                    target_time: trigger_time,
                },
            },
        )
        .unwrap();
    }

    VAULT_CACHE
        .save(deps.storage, &VaultCache { vault_id: vault.id })
        .unwrap();

    get_vault_handler(deps.as_ref(), vault.id).unwrap().vault
}

pub fn set_fin_price(deps: &mut OwnedDeps<MemoryStorage, MockApi, MockQuerier>, price: Decimal) {
    deps.querier.update_wasm(move |query| {
        SystemResult::Ok(ContractResult::Ok(match query {
            WasmQuery::Smart { msg, .. } => match from_binary(msg).unwrap() {
                QueryMsg::Simulation { offer_asset } => to_binary(&FinSimulationResponse {
                    return_amount: offer_asset.amount * (Decimal::one() / price.clone()),
                })
                .unwrap(),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }))
    });
}
