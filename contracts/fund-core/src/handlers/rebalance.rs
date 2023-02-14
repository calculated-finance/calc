use crate::{
    contract::{ContractResult, AFTER_FAILED_SWAP_REPLY_ID},
    shared::helpers::get_current_balance_values,
    state::get_config,
    types::failure_behaviour::FailureBehaviour,
    validation::{assert_allocations_sum_to_one, assert_sender_is_router},
};
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Decimal, Decimal256, Deps, Env, MessageInfo, ReplyOn, Response,
    SubMsg, Uint128, WasmMsg,
};
use std::collections::{HashMap, VecDeque};
use swapper::msg::ExecuteMsg;

pub fn rebalance_handler(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    allocations: &Vec<(String, Decimal)>,
    slippage_tolerance: Option<Decimal256>,
    failure_behaviour: Option<FailureBehaviour>,
) -> ContractResult<Response> {
    assert_sender_is_router(deps, info.sender)?;

    let new_allocations = allocations
        .iter()
        .map(|(denom, allocation)| (denom.clone(), *allocation))
        .collect::<HashMap<_, _>>();

    assert_allocations_sum_to_one(&new_allocations)?;

    let current_balances = deps
        .querier
        .query_all_balances(env.contract.address)?
        .into_iter()
        .map(|coin| (coin.denom.clone(), coin))
        .collect::<HashMap<_, _>>();

    let config = get_config(deps.storage)?;

    let current_balance_values = get_current_balance_values(deps, &current_balances)?;

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

    let mut over_allocations = VecDeque::<(&String, Uint128)>::new();
    let mut under_allocations = VecDeque::<(&String, Uint128)>::new();

    current_allocations
        .iter()
        .for_each(|(denom, current_allocation)| {
            let new_allocation = if new_allocations.contains_key(denom) {
                new_allocations[denom]
            } else {
                Decimal::zero()
            };

            let allocation_delta_value =
                (current_allocation).abs_diff(new_allocation) * total_fund_value;

            if current_allocation > &new_allocation {
                over_allocations.push_front((denom, allocation_delta_value))
            } else if current_allocation < &new_allocation {
                under_allocations.push_front((denom, allocation_delta_value))
            }
        });

    over_allocations
        .make_contiguous()
        .sort_by(|(_, allocation_a), (_, allocation_b)| allocation_a.cmp(allocation_b));

    let oa = over_allocations.clone();
    let ua = under_allocations.clone();

    let swap_messages = over_allocations
        .iter()
        .map(|(swap_denom, mut over_allocation_value)| {
            let mut swap_messages = Vec::<SubMsg>::new();

            while over_allocation_value > Uint128::zero() && !under_allocations.is_empty() {
                let (target_denom, under_allocation_value) = under_allocations
                    .pop_front()
                    .expect("next under allocation");

                let value_to_be_swapped = if over_allocation_value > under_allocation_value {
                    under_allocation_value
                } else {
                    over_allocation_value
                };

                let total_value_of_swap_denom = current_balance_values[*swap_denom];

                let current_balance_of_swap_denom = current_balances[*swap_denom].amount;

                let portion_of_swap_denom_to_send =
                    Decimal::from_ratio(value_to_be_swapped, total_value_of_swap_denom);

                let mut swap_amount = Coin::new(
                    (portion_of_swap_denom_to_send * current_balance_of_swap_denom).into(),
                    swap_denom.clone(),
                );

                if current_balance_of_swap_denom - swap_amount.amount < Uint128::new(50000) {
                    swap_amount =
                        Coin::new(current_balance_of_swap_denom.into(), swap_denom.clone());
                }

                if swap_amount.amount < Uint128::new(50000) {
                    continue;
                }

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

                over_allocation_value -= value_to_be_swapped;

                if over_allocation_value == Uint128::zero() {
                    under_allocations
                        .push_front((target_denom, under_allocation_value - value_to_be_swapped));
                }
            }

            swap_messages
        })
        .flatten()
        .collect::<Vec<SubMsg>>();

    Ok(Response::new()
        .add_submessages(swap_messages.clone())
        .add_attribute("new_allocations", format!("{:?}", new_allocations))
        .add_attribute("current_allocations", format!("{:?}", current_allocations))
        .add_attribute("current_balances", format!("{:?}", current_balances))
        .add_attribute(
            "current_balance_values",
            format!("{:?}", current_balance_values),
        )
        .add_attribute("has_failures", "false")
        .add_attribute("total_fund_value", format!("{:?}", total_fund_value))
        .add_attribute("over_allocations", format!("{:?}", oa))
        .add_attribute("under_allocations", format!("{:?}", ua))
        .add_attribute("swap_messages", format!("{:?}", swap_messages)))
}

pub fn after_failed_swap_handler() -> ContractResult<Response> {
    Ok(Response::new().add_attribute("has_failures", "true"))
}
