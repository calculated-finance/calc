use crate::{
    contract::{ContractResult, AFTER_FAILED_SWAP_REPLY_ID},
    state::get_config,
    types::failure_behaviour::FailureBehaviour,
    validation::{assert_allocations_sum_to_one, assert_sender_is_router},
};
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Decimal, Decimal256, Deps, Env, MessageInfo, ReplyOn, Response,
    SubMsg, Uint128, WasmMsg,
};
use std::collections::{HashMap, VecDeque};
use swapper::{msg::ExecuteMsg, shared::helpers::get_cheapest_swap_path};

pub fn rebalance_handler(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    new_allocations: &HashMap<String, Decimal>,
    slippage_tolerance: Option<Decimal256>,
    failure_behaviour: Option<FailureBehaviour>,
) -> ContractResult<Response> {
    assert_sender_is_router(deps, info.sender)?;
    assert_allocations_sum_to_one(new_allocations)?;

    let current_balances = deps
        .querier
        .query_all_balances(env.contract.address)?
        .into_iter()
        .map(|coin| (coin.denom.clone(), coin))
        .collect::<HashMap<_, _>>();

    let config = get_config(deps.storage)?;

    let current_balance_values = current_balances
        .values()
        .flat_map(|asset| {
            get_cheapest_swap_path(deps, asset, &config.base_asset).map(|path| {
                (
                    asset.denom.clone(),
                    asset.amount * (Decimal::one() / path.price),
                )
            })
        })
        .collect::<HashMap<_, _>>();

    let total_fund_value = current_balance_values
        .iter()
        .map(|(_, denom_value)| denom_value)
        .sum::<Uint128>();

    let current_allocations = current_balance_values
        .iter()
        .map(|(denom, denom_value)| {
            (
                denom.clone(),
                Decimal::from_ratio(*denom_value, total_fund_value),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut over_allocations = VecDeque::new();
    let mut under_allocations = VecDeque::new();

    current_allocations
        .iter()
        .for_each(|(denom, current_allocation)| {
            let new_allocation = new_allocations
                .get(denom)
                .expect(&format!("new allocation for {}", denom));

            let allocation_delta_value =
                (current_allocation).abs_diff(*new_allocation) * total_fund_value;

            if current_allocation > new_allocation {
                over_allocations.push_front((denom, allocation_delta_value))
            } else {
                under_allocations.push_front((denom, allocation_delta_value))
            }
        });

    let swap_messages = over_allocations
        .iter()
        .map(|(swap_denom, over_allocation_value)| {
            let mut swap_messages = Vec::<SubMsg>::new();

            let mut swap_denom_balance = current_balances
                .get(*swap_denom)
                .expect("swap denom balance")
                .clone();

            while swap_denom_balance.amount > Uint128::zero() && !under_allocations.is_empty() {
                let (target_denom, under_allocation_value) = under_allocations
                    .pop_front()
                    .expect("next under allocation");

                let value_to_be_swapped = if *over_allocation_value > under_allocation_value {
                    under_allocation_value
                } else {
                    *over_allocation_value
                };

                let total_value_of_swap_denom =
                    current_balance_values.get(*swap_denom).expect(&format!(
                        "total value of {} in terms of {}",
                        swap_denom, config.base_asset
                    ));

                let current_balance_of_swap_denom = current_balances
                    .get(*swap_denom)
                    .expect(&format!("current balance of {}", swap_denom))
                    .amount;

                let swap_amount = Coin::new(
                    (value_to_be_swapped / total_value_of_swap_denom
                        * current_balance_of_swap_denom)
                        .into(),
                    swap_denom.clone(),
                );

                if *over_allocation_value > under_allocation_value {
                    swap_denom_balance.amount -= swap_amount.amount;
                } else {
                    swap_denom_balance.amount = Uint128::zero();
                    under_allocations
                        .push_front((target_denom, under_allocation_value - value_to_be_swapped));
                };

                swap_messages.push(SubMsg {
                    id: AFTER_FAILED_SWAP_REPLY_ID,
                    msg: CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.swapper.to_string(),
                        msg: to_binary(&ExecuteMsg::CreateSwap {
                            target_denom: target_denom.clone(),
                            slippage_tolerance,
                            on_complete: None,
                        })
                        .expect("message binary"),
                        funds: vec![swap_amount],
                    }),
                    gas_limit: None,
                    reply_on: match failure_behaviour
                        .as_ref()
                        .unwrap_or(&FailureBehaviour::BestEffort)
                    {
                        FailureBehaviour::BestEffort => ReplyOn::Error,
                        FailureBehaviour::Rollback => ReplyOn::Never,
                    },
                });
            }

            swap_messages
        })
        .flatten()
        .collect::<Vec<SubMsg>>();

    Ok(Response::new().add_submessages(swap_messages))
}

pub fn after_failed_swap_handler() -> ContractResult<Response> {
    Ok(Response::new().add_attribute("has_failures", "true"))
}
