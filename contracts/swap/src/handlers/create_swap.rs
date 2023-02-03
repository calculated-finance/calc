use crate::{
    msg::ExecuteMsg,
    state::{
        messages::{get_next_swap_id, save_messages},
        paths::get_path,
    },
    types::{callback::Callback, pair::Pair},
    validation::assert_exactly_one_asset,
};
use base::pair::Pair as FinPair;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal256, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, WasmMsg,
};
use std::collections::VecDeque;

pub fn create_swap_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target_denom: String,
    slippage_tolerance: Option<Decimal256>,
    on_complete: Callback,
) -> StdResult<Response> {
    assert_exactly_one_asset(info.funds.clone())?;

    let swap_amount = info.funds[0].clone();
    let swap_denom = swap_amount.denom.clone();

    let swap_path = get_path(deps.storage, [swap_denom.clone(), target_denom.clone()])?;

    if swap_path.is_empty() {
        return Err(StdError::GenericErr {
            msg: format!("no path found between {} and {}", swap_denom, target_denom),
        });
    }

    let swap_id = get_next_swap_id(deps.storage)?;
    let callback = to_binary(&ExecuteMsg::ContinueSwap { swap_id })?;

    let mut messages = swap_path
        .iter()
        .map(|pair| {
            generate_swap_message(
                env.clone(),
                pair.clone(),
                slippage_tolerance,
                callback.clone(),
            )
        })
        .flatten()
        .collect::<VecDeque<Callback>>();

    messages.push_back(on_complete);
    save_messages(deps.storage, swap_id, messages)?;

    Ok(Response::new()
        .add_attribute("method", "swap")
        .add_attribute("swap_id", swap_id.to_string())
        .add_attribute(
            "path",
            format!(
                "[{}]",
                swap_path
                    .iter()
                    .map(|p| format!("{:?}", p))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        )
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: callback,
            funds: vec![swap_amount],
        })))
}

fn generate_swap_message(
    env: Env,
    pair: Pair,
    slippage_tolerance: Option<Decimal256>,
    callback: Binary,
) -> StdResult<Callback> {
    match pair {
        Pair::Fin {
            address,
            base_denom,
            quote_denom,
        } => Ok(Callback {
            address: env.contract.address,
            msg: to_binary(&ExecuteMsg::SwapOnFin {
                pair: FinPair {
                    address,
                    base_denom,
                    quote_denom,
                },
                slippage_tolerance,
                callback,
            })?,
        }),
    }
}

#[cfg(test)]
mod swap_tests {
    use super::*;
    use crate::state::paths::add_path;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Attribute, Coin, SubMsg, Uint128,
    };

    #[test]
    fn swap_with_no_swap_asset_should_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[]);

        let response = create_swap_handler(
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
            "Generic error: received 0 denoms but required exactly 1"
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

        let response = create_swap_handler(
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
            "Generic error: no path found between swap_denom and target_denom"
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

        let pair = FinPair {
            address: Addr::unchecked("fin_pair_1"),
            quote_denom: "swap_denom".to_string(),
            base_denom: "transfer_denom".to_string(),
        };

        add_path(
            deps.as_mut().storage,
            ["swap_denom".to_string(), "transfer_denom".to_string()],
            Pair::Fin {
                address: pair.address.clone(),
                quote_denom: pair.quote_denom.clone(),
                base_denom: pair.base_denom.clone(),
            },
        )
        .unwrap();

        add_path(
            deps.as_mut().storage,
            ["transfer_denom".to_string(), "target_denom".to_string()],
            Pair::Fin {
                address: Addr::unchecked("fin_pair_2"),
                quote_denom: "transfer_denom".to_string(),
                base_denom: "target_denom".to_string(),
            },
        )
        .unwrap();

        let response = create_swap_handler(
            deps.as_mut(),
            env.clone(),
            info,
            "target_denom".to_string(),
            None,
            Callback {
                address: Addr::unchecked("sender"),
                msg: to_binary("callback").unwrap(),
            },
        );

        assert!(response
            .unwrap()
            .messages
            .contains(&SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::ContinueSwap { swap_id: 1 }).unwrap(),
                funds: vec![swap_amount]
            }))));
    }

    #[test]
    fn swap_with_fin_path_should_return_swap_id() {
        let swap_amount = Coin {
            denom: "swap_denom".to_string(),
            amount: Uint128::new(1000000),
        };

        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[swap_amount.clone()]);

        add_path(
            deps.as_mut().storage,
            ["swap_denom".to_string(), "transfer_denom".to_string()],
            Pair::Fin {
                address: Addr::unchecked("fin_pair_1"),
                quote_denom: "swap_denom".to_string(),
                base_denom: "transfer_denom".to_string(),
            },
        )
        .unwrap();

        add_path(
            deps.as_mut().storage,
            ["transfer_denom".to_string(), "target_denom".to_string()],
            Pair::Fin {
                address: Addr::unchecked("fin_pair_2"),
                quote_denom: "transfer_denom".to_string(),
                base_denom: "target_denom".to_string(),
            },
        )
        .unwrap();

        let response = create_swap_handler(
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

        assert!(response
            .unwrap()
            .attributes
            .contains(&Attribute::new("swap_id", "1")));
    }

    #[test]
    fn swap_with_fin_path_should_return_path() {
        let swap_amount = Coin {
            denom: "swap_denom".to_string(),
            amount: Uint128::new(1000000),
        };

        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("admin", &[swap_amount.clone()]);

        add_path(
            deps.as_mut().storage,
            ["swap_denom".to_string(), "transfer_denom".to_string()],
            Pair::Fin {
                address: Addr::unchecked("fin_pair_1"),
                quote_denom: "swap_denom".to_string(),
                base_denom: "transfer_denom".to_string(),
            },
        )
        .unwrap();

        add_path(
            deps.as_mut().storage,
            ["transfer_denom".to_string(), "target_denom".to_string()],
            Pair::Fin {
                address: Addr::unchecked("fin_pair_2"),
                quote_denom: "transfer_denom".to_string(),
                base_denom: "target_denom".to_string(),
            },
        )
        .unwrap();

        let response = create_swap_handler(
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

        assert!(response
            .unwrap()
            .attributes
            .contains(&Attribute::new("path", "[Fin { address: Addr(\"fin_pair_1\"), quote_denom: \"swap_denom\", base_denom: \"transfer_denom\" }, Fin { address: Addr(\"fin_pair_2\"), quote_denom: \"transfer_denom\", base_denom: \"target_denom\" }]")));
    }
}
