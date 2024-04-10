use astroport::router::ExecuteMsg;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, SubMsg,
};
use shared::{
    coin::subtract,
    cw20::{into_bank_msg, into_execute_msg},
};

use crate::{
    contract::AFTER_SWAP,
    state::{
        cache::{SwapCache, SWAP_CACHE},
        config::get_config,
    },
    ContractError,
};

pub fn swap_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minimum_receive_amount: Coin,
    route: Binary,
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::AssetCardinality {});
    }

    let swap_amount = info.funds[0].clone();

    if swap_amount.amount.is_zero() {
        return Err(ContractError::SwapAmount {});
    }

    let config = get_config(deps.storage)?;

    let swap_operations = from_json(route)?;

    SWAP_CACHE.save(
        deps.storage,
        &SwapCache {
            sender: info.sender.clone(),
            minimum_receive_amount: minimum_receive_amount.clone(),
            target_denom_balance: deps.querier.query_balance(
                env.contract.address.clone(),
                minimum_receive_amount.denom.clone(),
            )?,
        },
    )?;

    Ok(Response::new()
        .add_attribute("swap", "true")
        .add_attribute("sender", info.sender)
        .add_attribute("swap_amount", swap_amount.to_string())
        .add_attribute("minimum_receive_amount", minimum_receive_amount.to_string())
        .add_submessage(SubMsg::reply_on_success(
            into_execute_msg(
                deps.api,
                config.router_address,
                to_json_binary(&ExecuteMsg::ExecuteSwapOperations {
                    operations: swap_operations,
                    minimum_receive: Some(minimum_receive_amount.amount),
                    to: None,
                    max_spread: None,
                })?,
                swap_amount,
            )?,
            AFTER_SWAP,
        )))
}

pub fn return_swapped_funds(deps: Deps, env: Env) -> Result<Response, ContractError> {
    let swap_cache = SWAP_CACHE.load(deps.storage)?;

    let updated_target_denom_balance = deps.querier.query_balance(
        env.contract.address,
        swap_cache.minimum_receive_amount.denom.clone(),
    )?;

    let return_amount = subtract(
        &updated_target_denom_balance,
        &swap_cache.target_denom_balance,
    )?;

    if return_amount.amount < swap_cache.minimum_receive_amount.amount {
        return Err(ContractError::ReceiveAmount {});
    }

    Ok(Response::new()
        .add_attribute("return_amount", return_amount.to_string())
        .add_submessage(SubMsg::new(into_bank_msg(
            deps.api,
            swap_cache.sender.as_ref(),
            vec![return_amount],
        )?)))
}

#[cfg(test)]
mod swap_tests {
    use astroport::{
        asset::AssetInfo,
        router::{ExecuteMsg, SwapOperation},
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        to_json_binary, Addr, Coin, CosmosMsg, ReplyOn, SubMsg, WasmMsg,
    };

    use crate::{
        contract::AFTER_SWAP,
        handlers::swap::swap_handler,
        state::{cache::SWAP_CACHE, config::update_config},
        tests::constants::{ADMIN, DENOM_UATOM, DENOM_UOSMO, DENOM_USDC, ROUTER},
        types::config::Config,
        ContractError,
    };

    #[test]
    fn with_no_assets_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[]),
                Coin::new(12312, DENOM_UOSMO),
                to_json_binary("route").unwrap()
            )
            .unwrap_err(),
            ContractError::AssetCardinality {}
        )
    }

    #[test]
    fn with_multiple_assets_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(
                    ADMIN,
                    &[Coin::new(12312, DENOM_UATOM), Coin::new(12312, DENOM_UOSMO)]
                ),
                Coin::new(12312, DENOM_UOSMO),
                to_json_binary("route").unwrap()
            )
            .unwrap_err(),
            ContractError::AssetCardinality {}
        )
    }

    #[test]
    fn with_zero_swap_amount_fails() {
        assert_eq!(
            swap_handler(
                mock_dependencies().as_mut(),
                mock_env(),
                mock_info(ADMIN, &[Coin::new(0, DENOM_UATOM)]),
                Coin::new(12312, DENOM_UATOM),
                to_json_binary("route").unwrap()
            )
            .unwrap_err(),
            ContractError::SwapAmount {}
        )
    }

    #[test]
    fn caches_details_correctly() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![Coin::new(0, DENOM_UATOM)],
        );

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                router_address: Addr::unchecked(ROUTER),
            },
        )
        .unwrap();

        let info = mock_info(ADMIN, &[Coin::new(2347631, DENOM_UATOM)]);
        let minimum_receive_amount = Coin::new(3873213, DENOM_USDC);

        let route = vec![SwapOperation::AstroSwap {
            offer_asset_info: AssetInfo::NativeToken {
                denom: DENOM_UATOM.to_string(),
            },
            ask_asset_info: AssetInfo::Token {
                contract_addr: Addr::unchecked(DENOM_USDC),
            },
        }];

        swap_handler(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            minimum_receive_amount.clone(),
            to_json_binary(&route).unwrap(),
        )
        .unwrap();

        let swap_cache = SWAP_CACHE.load(deps.as_ref().storage).unwrap();

        assert_eq!(swap_cache.sender, ADMIN);
        assert_eq!(
            swap_cache.target_denom_balance,
            deps.as_ref()
                .querier
                .query_balance(env.contract.address, minimum_receive_amount.denom.clone())
                .unwrap()
        );
        assert_eq!(swap_cache.minimum_receive_amount, minimum_receive_amount);
    }

    #[test]
    fn sends_swap_message() {
        let mut deps = mock_dependencies();

        let info = mock_info(ADMIN, &[Coin::new(2347631, DENOM_UATOM)]);

        let minimum_receive_amount = Coin::new(3873213, DENOM_USDC);

        update_config(
            deps.as_mut().storage,
            Config {
                admin: Addr::unchecked(ADMIN),
                router_address: Addr::unchecked(ROUTER),
            },
        )
        .unwrap();

        let route = vec![SwapOperation::AstroSwap {
            offer_asset_info: AssetInfo::NativeToken {
                denom: DENOM_UATOM.to_string(),
            },
            ask_asset_info: AssetInfo::Token {
                contract_addr: Addr::unchecked(DENOM_USDC),
            },
        }];

        let response = swap_handler(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            minimum_receive_amount.clone(),
            to_json_binary(&route).unwrap(),
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages.first().unwrap(),
            &SubMsg {
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ROUTER.to_string(),
                    msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperations {
                        operations: route,
                        minimum_receive: Some(minimum_receive_amount.amount),
                        to: None,
                        max_spread: None,
                    })
                    .unwrap(),
                    funds: vec![Coin::new(2347631, DENOM_UATOM)],
                }),
                id: AFTER_SWAP,
                reply_on: ReplyOn::Success,
                gas_limit: None,
            }
        )
    }
}

#[cfg(test)]
mod return_swapped_funds_tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, BankMsg, Coin,
    };
    use shared::coin::add;

    use crate::{
        handlers::swap::return_swapped_funds,
        state::cache::{SwapCache, SWAP_CACHE},
        tests::constants::DENOM_UOSMO,
        ContractError,
    };

    #[test]
    fn with_return_amount_smaller_than_minimum_receive_amount_fails() {
        let mut deps = mock_dependencies();

        let minimum_receive_amount = Coin::new(123, DENOM_UOSMO);

        let swap_cache = SwapCache {
            sender: Addr::unchecked("sender"),
            minimum_receive_amount: minimum_receive_amount.clone(),
            target_denom_balance: Coin::new(122, DENOM_UOSMO),
        };

        SWAP_CACHE.save(deps.as_mut().storage, &swap_cache).unwrap();

        assert_eq!(
            return_swapped_funds(deps.as_ref(), mock_env()).unwrap_err(),
            ContractError::ReceiveAmount {}
        )
    }

    #[test]
    fn sends_funds_back_to_sender() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let minimum_receive_amount = Coin::new(123, DENOM_UOSMO);
        let target_denom_balance = Coin::new(122, DENOM_UOSMO);
        let return_amount = Coin::new(153, DENOM_UOSMO);

        let swap_cache = SwapCache {
            sender: Addr::unchecked("sender"),
            minimum_receive_amount,
            target_denom_balance: target_denom_balance.clone(),
        };

        SWAP_CACHE.save(deps.as_mut().storage, &swap_cache).unwrap();

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![add(target_denom_balance, return_amount.clone()).unwrap()],
        );

        let response = return_swapped_funds(deps.as_ref(), env).unwrap();

        assert_eq!(
            response.messages.first().unwrap(),
            &cosmwasm_std::SubMsg::new(BankMsg::Send {
                to_address: swap_cache.sender.to_string(),
                amount: vec![return_amount],
            })
        )
    }
}
