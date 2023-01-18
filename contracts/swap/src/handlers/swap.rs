use crate::{
    contract::{AFTER_FIN_SWAP_REPLY_ID, AFTER_SWAP_CALLBACK_INVOKED_ID},
    errors::contract_error::ContractError,
    state::{
        cache::{SwapCache, SWAP_CACHE},
        paths::get_path,
        swaps::{delete_swap, get_swap, save_swap},
    },
    types::{callback::Callback, pair::Pair, swap::Swap},
    validation::assert_exactly_one_asset,
};
use base::pair::Pair as FINPair;
use cosmwasm_std::{
    Coin, CosmosMsg, Decimal256, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg,
    SubMsgResult, WasmMsg,
};
use fin_helpers::swaps::create_fin_swap_message;

pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target_denom: String,
    slippage_tolerance: Option<Decimal256>,
    callback: Callback,
) -> Result<Response, ContractError> {
    assert_exactly_one_asset(info.funds.clone())?;

    let swap_denom = info.funds[0].denom.clone();
    let path = get_path(deps.storage, [swap_denom.clone(), target_denom.clone()])?;

    if path.len() == 0 {
        return Err(ContractError::CustomError {
            val: format!("no path found between {} and {}", swap_denom, target_denom),
        });
    }

    let swap_id = save_swap(
        deps.storage,
        Swap {
            path: path.clone(),
            callback,
        },
    )?;

    let starting_swap_message = generate_swap_message(
        deps,
        env,
        swap_id,
        path[0].clone(),
        info.funds[0].clone(),
        slippage_tolerance,
    )?;

    Ok(Response::new()
        .add_attribute("method", "swap")
        .add_submessage(starting_swap_message))
}

fn generate_swap_message(
    deps: DepsMut,
    env: Env,
    swap_id: u64,
    pair: Pair,
    swap_amount: Coin,
    slippage_tolerance: Option<Decimal256>,
) -> StdResult<SubMsg> {
    match pair {
        Pair::Fin {
            address,
            base_denom,
            quote_denom,
        } => {
            SWAP_CACHE.save(
                deps.storage,
                &SwapCache {
                    swap_id,
                    swap_denom_balance: deps
                        .querier
                        .query_balance(&env.contract.address, &swap_amount.denom)?,
                    receive_denom_balance: deps.querier.query_balance(
                        &env.contract.address,
                        match base_denom == swap_amount.denom {
                            true => quote_denom.clone(),
                            false => base_denom.clone(),
                        },
                    )?,
                },
            )?;

            Ok(create_fin_swap_message(
                deps.querier,
                FINPair {
                    address,
                    base_denom,
                    quote_denom,
                },
                swap_amount,
                slippage_tolerance,
                AFTER_FIN_SWAP_REPLY_ID,
            )?)
        }
    }
}

pub fn after_fin_swap(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let swap_cache = SWAP_CACHE.load(deps.storage)?;
    let mut swap = get_swap(deps.storage, swap_cache.swap_id)?;

    let receive_denom_balance = deps.querier.query_balance(
        &env.contract.address,
        &swap_cache.receive_denom_balance.denom,
    )?;

    swap.path = swap.path[1..].to_vec();

    let message = match swap.path.len() {
        0 => SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: swap.callback.address.to_string(),
                msg: swap.callback.msg,
                funds: vec![receive_denom_balance],
            }),
            AFTER_SWAP_CALLBACK_INVOKED_ID,
        ),
        _ => generate_swap_message(
            deps,
            env,
            swap_cache.swap_id,
            swap.path[0].clone(),
            receive_denom_balance,
            None,
        )?,
    };

    Ok(Response::new().add_submessage(message))
}

pub fn after_swap_callback_invoked(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let swap_cache = SWAP_CACHE.load(deps.storage)?;

    match reply.result {
        SubMsgResult::Ok(_) => {
            delete_swap(deps.storage, swap_cache.swap_id)?;
        }
        SubMsgResult::Err(_) => {
            return Err(ContractError::CustomError {
                val: "swap callback invocation failed. You can update the swap callback and try again by calling XYZ.".to_string(),
            })
        }
    }

    SWAP_CACHE.remove(deps.storage);
    Ok(Response::new().add_attribute("callback", "success"))
}

#[cfg(test)]
mod swap_tests {
    use super::*;
    use crate::{contract::AFTER_FIN_SWAP_REPLY_ID, state::paths::add_path};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        to_binary, Addr, Uint128,
    };
    use kujira::fin::ExecuteMsg as FINExecuteMsg;

    #[test]
    fn swap_with_no_swap_asset_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[]);

        let response = swap(
            deps.as_mut(),
            env,
            info,
            "target_denom".to_string(),
            None,
            Callback {
                address: Addr::unchecked("sender"),
                msg: to_binary("callback").unwrap(),
            },
        );

        assert_eq!(
            response.unwrap_err().to_string(),
            "Error: received 0 denoms but required exactly 1"
        )
    }

    #[test]
    fn swap_with_no_path_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(
            "admin",
            &[Coin {
                denom: "swap_denom".to_string(),
                amount: Uint128::new(1000000),
            }],
        );

        let response = swap(
            deps.as_mut(),
            env,
            info,
            "target_denom".to_string(),
            None,
            Callback {
                address: Addr::unchecked("sender"),
                msg: to_binary("callback").unwrap(),
            },
        );

        assert_eq!(
            response.unwrap_err().to_string(),
            "Error: no path found between swap_denom and target_denom"
        )
    }

    #[test]
    fn swap_with_fin_path_should_invoke_fin_swap() {
        let swap_amount = Coin {
            denom: "swap_denom".to_string(),
            amount: Uint128::new(1000000),
        };

        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[swap_amount.clone()]);

        add_path(
            deps.as_mut().storage,
            ["swap_denom".to_string(), "target_denom".to_string()],
            Pair::Fin {
                address: Addr::unchecked("fin_pair"),
                quote_denom: "swap_denom".to_string(),
                base_denom: "target_denom".to_string(),
            },
        )
        .unwrap();

        let response = swap(
            deps.as_mut(),
            env,
            info,
            "target_denom".to_string(),
            None,
            Callback {
                address: Addr::unchecked("sender"),
                msg: to_binary("callback").unwrap(),
            },
        );

        assert!(response.unwrap().messages.contains(&SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "fin_pair".to_string(),
                msg: to_binary(&FINExecuteMsg::Swap {
                    offer_asset: None,
                    belief_price: None,
                    max_spread: None,
                    to: None
                })
                .unwrap(),
                funds: vec![swap_amount]
            }),
            AFTER_FIN_SWAP_REPLY_ID
        )));
    }
}

#[cfg(test)]
mod after_fin_swap_tests {
    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        to_binary, Addr, Uint128,
    };
    use kujira::fin::ExecuteMsg as FINExecuteMsg;

    #[test]
    fn with_no_more_swaps_should_invoke_callback() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let swap = Swap {
            path: vec![Pair::Fin {
                address: Addr::unchecked("fin_pair"),
                quote_denom: "swap_denom".to_string(),
                base_denom: "target_denom".to_string(),
            }],
            callback: Callback {
                address: Addr::unchecked("sender"),
                msg: to_binary("callback").unwrap(),
            },
        };

        let swap_id = save_swap(deps.as_mut().storage, swap).unwrap();

        let swap_amount = Coin {
            denom: "swap_denom".to_string(),
            amount: Uint128::new(1000000),
        };

        let received_amount = Coin {
            denom: "target_denom".to_string(),
            amount: Uint128::new(1000000),
        };

        SWAP_CACHE
            .save(
                deps.as_mut().storage,
                &SwapCache {
                    swap_id,
                    swap_denom_balance: swap_amount.clone(),
                    receive_denom_balance: Coin::new(0, "target_denom"),
                },
            )
            .unwrap();

        deps.querier.update_balance(
            "cosmos2contract",
            vec![Coin::new(0, "swap_denom"), received_amount.clone()],
        );

        let response = after_fin_swap(deps.as_mut(), env);

        assert!(response.unwrap().messages.contains(&SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "sender".to_string(),
                msg: to_binary("callback").unwrap(),
                funds: vec![received_amount]
            }),
            AFTER_SWAP_CALLBACK_INVOKED_ID
        )));
    }

    #[test]
    fn with_more_swaps_should_invoke_next_swap() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let swap = Swap {
            path: vec![
                Pair::Fin {
                    address: Addr::unchecked("fin_pair_1"),
                    quote_denom: "swap_denom".to_string(),
                    base_denom: "transfer_denom".to_string(),
                },
                Pair::Fin {
                    address: Addr::unchecked("fin_pair_2"),
                    quote_denom: "transfer_denom".to_string(),
                    base_denom: "target_denom".to_string(),
                },
            ],
            callback: Callback {
                address: Addr::unchecked("sender"),
                msg: to_binary("callback").unwrap(),
            },
        };

        let swap_id = save_swap(deps.as_mut().storage, swap).unwrap();

        let swap_amount = Coin {
            denom: "swap_denom".to_string(),
            amount: Uint128::new(1000000),
        };

        let received_amount = Coin {
            denom: "transfer_denom".to_string(),
            amount: Uint128::new(1000000),
        };

        SWAP_CACHE
            .save(
                deps.as_mut().storage,
                &SwapCache {
                    swap_id,
                    swap_denom_balance: swap_amount.clone(),
                    receive_denom_balance: Coin::new(0, "transfer_denom"),
                },
            )
            .unwrap();

        deps.querier.update_balance(
            "cosmos2contract",
            vec![Coin::new(0, "swap_denom"), received_amount.clone()],
        );

        let response = after_fin_swap(deps.as_mut(), env);

        assert!(response.unwrap().messages.contains(&SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "fin_pair_2".to_string(),
                msg: to_binary(&FINExecuteMsg::Swap {
                    offer_asset: None,
                    belief_price: None,
                    max_spread: None,
                    to: None
                })
                .unwrap(),
                funds: vec![received_amount]
            }),
            AFTER_FIN_SWAP_REPLY_ID
        )));
    }
}
