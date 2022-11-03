use crate::constants::{ONE, ONE_THOUSAND};
use crate::contract::reply;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VaultResponse};
use crate::vault::Vault;
use base::helpers::message_helpers::get_flat_map_for_event_type;
use base::vaults::vault::Destination;
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, Decimal, Decimal256, Empty, Env, Event, MessageInfo,
    Response, StdError, StdResult, Uint128, Uint256, Uint64,
};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use fin_helpers::codes::ERROR_SWAP_SLIPPAGE_EXCEEDED;
use kujira::denom::Denom;
use kujira::fin::{
    BookResponse, ExecuteMsg as FINExecuteMsg, InstantiateMsg as FINInstantiateMsg, OrderResponse,
    PoolResponse, QueryMsg as FINQueryMsg,
};
use rand::Rng;
use std::collections::HashMap;
use std::str::FromStr;

pub const USER: &str = "user";
pub const ADMIN: &str = "admin";
pub const DENOM_UKUJI: &str = "ukuji";
pub const DENOM_UTEST: &str = "utest";

pub struct MockApp {
    pub app: App,
    pub dca_contract_address: Addr,
    pub fin_contract_address: Addr,
    pub vault_ids: HashMap<String, Uint128>,
    pub fee_percent: Decimal,
}

impl MockApp {
    pub fn new(fin_contract: Box<dyn Contract<Empty>>) -> Self {
        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![
                        Coin {
                            denom: String::from(DENOM_UKUJI),
                            amount: ONE_THOUSAND,
                        },
                        Coin {
                            denom: String::from(DENOM_UTEST),
                            amount: ONE_THOUSAND,
                        },
                    ],
                )
                .unwrap();
        });

        let dca_contract_address = Self::instantiate_contract(
            &mut app,
            Box::new(
                ContractWrapper::new(
                    crate::contract::execute,
                    crate::contract::instantiate,
                    crate::contract::query,
                )
                .with_reply(reply),
            ),
            Addr::unchecked(ADMIN),
            &InstantiateMsg {
                admin: Addr::unchecked(ADMIN),
                fee_collector: Addr::unchecked(ADMIN),
                fee_percent: Decimal::from_str("0.015").unwrap(),
                staking_router_address: Addr::unchecked("staking-router"),
                page_limit: 1000,
                minumum_execution_interval_in_seconds: 5 * 60,
            },
            "dca",
        );

        let fin_contract_address = Self::instantiate_contract(
            &mut app,
            fin_contract,
            Addr::unchecked(ADMIN),
            &FINInstantiateMsg {
                decimal_delta: None,
                denoms: [Denom::from(DENOM_UTEST), Denom::from(DENOM_UKUJI)],
                owner: Addr::unchecked(ADMIN),
                price_precision: kujira::precision::Precision::DecimalPlaces(3),
            },
            "fin",
        );

        app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &dca_contract_address,
                    vec![
                        Coin {
                            denom: String::from(DENOM_UKUJI),
                            amount: ONE_THOUSAND,
                        },
                        Coin {
                            denom: String::from(DENOM_UTEST),
                            amount: ONE_THOUSAND,
                        },
                    ],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &fin_contract_address,
                    vec![
                        Coin {
                            denom: String::from(DENOM_UKUJI),
                            amount: ONE_THOUSAND,
                        },
                        Coin {
                            denom: String::from(DENOM_UTEST),
                            amount: ONE_THOUSAND,
                        },
                    ],
                )
                .unwrap();
        });

        app.execute_contract(
            Addr::unchecked(ADMIN),
            dca_contract_address.clone(),
            &ExecuteMsg::CreatePair {
                address: fin_contract_address.clone(),
                base_denom: DENOM_UTEST.to_string(),
                quote_denom: DENOM_UKUJI.to_string(),
            },
            &[],
        )
        .unwrap();

        Self {
            app,
            dca_contract_address,
            fin_contract_address,
            vault_ids: HashMap::new(),
            fee_percent: Decimal::from_str("0.015").unwrap(),
        }
    }

    fn instantiate_contract<T: Serialize>(
        app: &mut App,
        contract: Box<dyn Contract<Empty>>,
        sender: Addr,
        msg: &T,
        label: &str,
    ) -> Addr {
        let code_id = app.store_code(contract);
        let contract_address = app
            .instantiate_contract(code_id, sender, msg, &[], label, None)
            .unwrap();

        contract_address
    }

    pub fn with_funds_for(mut self, address: &Addr, amount: Uint128, denom: &str) -> MockApp {
        self.app
            .send_tokens(
                Addr::unchecked(ADMIN),
                address.clone(),
                &[Coin::new(amount.into(), denom.to_string())],
            )
            .unwrap();

        self
    }

    pub fn with_vault_with_unfilled_fin_limit_price_trigger(
        mut self,
        owner: &Addr,
        destinations: Option<Vec<Destination>>,
        balance: Coin,
        swap_amount: Uint128,
        label: &str,
    ) -> Self {
        let response = self
            .app
            .execute_contract(
                owner.clone(),
                self.dca_contract_address.clone(),
                &ExecuteMsg::CreateVault {
                    owner: None,
                    price_threshold: None,
                    label: Some("label".to_string()),
                    destinations,
                    pair_address: self.fin_contract_address.clone(),
                    position_type: None,
                    slippage_tolerance: None,
                    swap_amount,
                    target_price: Some(Decimal256::from_str("1.0").unwrap()),
                    target_start_time_utc_seconds: None,
                    schedule_expression: "$ $ * ? * *".to_string(),
                },
                &vec![balance],
            )
            .unwrap();

        self.vault_ids.insert(
            String::from(label),
            Uint128::from_str(
                &get_flat_map_for_event_type(&response.events, "wasm").unwrap()["vault_id"],
            )
            .unwrap(),
        );

        self
    }

    pub fn with_vault_with_filled_fin_limit_price_trigger(
        mut self,
        owner: &Addr,
        destinations: Option<Vec<Destination>>,
        balance: Coin,
        swap_amount: Uint128,
        label: &str,
    ) -> Self {
        let response = self
            .app
            .execute_contract(
                owner.clone(),
                self.dca_contract_address.clone(),
                &ExecuteMsg::CreateVault {
                    owner: None,
                    price_threshold: None,
                    label: Some("label".to_string()),
                    destinations,
                    pair_address: self.fin_contract_address.clone(),
                    position_type: None,
                    slippage_tolerance: None,
                    swap_amount,
                    target_price: Some(Decimal256::from_str("1.0").unwrap()),
                    target_start_time_utc_seconds: None,
                    schedule_expression: "$ $ * ? * *".to_string(),
                },
                &vec![balance],
            )
            .unwrap();

        self.vault_ids.insert(
            String::from(label),
            Uint128::from_str(
                &get_flat_map_for_event_type(&response.events, "wasm").unwrap()["vault_id"],
            )
            .unwrap(),
        );

        // send swap amount after fees of ukuji out of fin contract to simulate outgoing
        self.app
            .send_tokens(
                self.fin_contract_address.clone(),
                Addr::unchecked(ADMIN),
                &[Coin::new(swap_amount.into(), DENOM_UKUJI)],
            )
            .unwrap();

        // send swap amount after fees of utest into fin contract to simulate incoming
        self.app
            .send_tokens(
                Addr::unchecked(ADMIN),
                self.fin_contract_address.clone(),
                &[Coin::new(swap_amount.into(), DENOM_UTEST)],
            )
            .unwrap();

        self
    }

    pub fn with_vault_with_partially_filled_fin_limit_price_trigger(
        mut self,
        owner: &Addr,
        balance: Coin,
        swap_amount: Uint128,
        label: &str,
    ) -> MockApp {
        let response = self
            .app
            .execute_contract(
                owner.clone(),
                self.dca_contract_address.clone(),
                &ExecuteMsg::CreateVault {
                    owner: None,
                    price_threshold: None,
                    label: Some("label".to_string()),
                    destinations: None,
                    pair_address: self.fin_contract_address.clone(),
                    position_type: None,
                    slippage_tolerance: None,
                    swap_amount,
                    target_price: Some(Decimal256::from_str("1.0").unwrap()),
                    target_start_time_utc_seconds: None,
                    schedule_expression: "$ $ * ? * *".to_string(),
                },
                &vec![balance],
            )
            .unwrap();

        self.vault_ids.insert(
            String::from(label),
            Uint128::from_str(
                &get_flat_map_for_event_type(&response.events, "wasm").unwrap()["vault_id"],
            )
            .unwrap(),
        );

        // send half swap amount ukuji from fin to admin wallet to mock partially filled outgoing
        self.app
            .send_tokens(
                self.fin_contract_address.clone(),
                Addr::unchecked(ADMIN),
                &[Coin {
                    denom: String::from(DENOM_UKUJI),
                    amount: swap_amount / Uint128::new(2),
                }],
            )
            .unwrap();

        // send half swap amount utest from admin wallet to fin to mock partially filled incoming
        self.app
            .send_tokens(
                Addr::unchecked(ADMIN),
                self.fin_contract_address.clone(),
                &[Coin {
                    denom: String::from(DENOM_UTEST),
                    amount: swap_amount / Uint128::new(2),
                }],
            )
            .unwrap();

        self
    }

    pub fn with_vault_with_time_trigger(
        mut self,
        owner: &Addr,
        destinations: Option<Vec<Destination>>,
        balance: Coin,
        swap_amount: Uint128,
        label: &str,
        price_threshold: Option<Decimal256>,
    ) -> MockApp {
        let response = self
            .app
            .execute_contract(
                owner.clone(),
                self.dca_contract_address.clone(),
                &ExecuteMsg::CreateVault {
                    owner: None,
                    price_threshold,
                    label: Some("label".to_string()),
                    destinations,
                    pair_address: self.fin_contract_address.clone(),
                    position_type: None,
                    slippage_tolerance: None,
                    swap_amount,
                    target_start_time_utc_seconds: Some(Uint64::from(
                        self.app.block_info().time.plus_seconds(2).seconds(),
                    )),
                    target_price: None,
                    schedule_expression: "$ $ * ? * *".to_string(),
                },
                &vec![balance],
            )
            .unwrap();

        self.vault_ids.insert(
            String::from(label),
            Uint128::from_str(
                &get_flat_map_for_event_type(&response.events, "wasm").unwrap()["vault_id"],
            )
            .unwrap(),
        );

        self
    }

    pub fn elapse_time(&mut self, seconds: u64) {
        self.app.update_block(|mut block_info| {
            block_info.time = block_info.time.plus_seconds(seconds);
            let seconds_per_block = 5u64;
            block_info.height += seconds / seconds_per_block;
        });
    }

    pub fn get_vault_by_label(&self, label: &str, address: Addr) -> Vault {
        let vault_id = self.vault_ids.get(label).unwrap();
        let vault_response: VaultResponse = self
            .app
            .wrap()
            .query_wasm_smart(
                self.dca_contract_address.clone(),
                &QueryMsg::GetVault {
                    vault_id: vault_id.to_owned(),
                    address,
                },
            )
            .unwrap();

        vault_response.vault
    }

    pub fn get_balance(&self, address: &Addr, denom: &str) -> Uint128 {
        self.app
            .wrap()
            .query_balance(address.clone(), denom)
            .unwrap()
            .amount
    }
}

fn default_swap_handler(info: MessageInfo) -> StdResult<Response> {
    let received_coin = info.funds[0].clone();
    let coin_to_send = match received_coin.denom.as_str() {
        DENOM_UKUJI => Coin {
            denom: String::from(DENOM_UTEST),
            amount: received_coin.amount,
        },
        DENOM_UTEST => Coin {
            denom: String::from(DENOM_UKUJI),
            amount: received_coin.amount,
        },
        _ => Coin {
            denom: String::from(DENOM_UTEST),
            amount: received_coin.amount,
        },
    };

    Ok(Response::new()
        .add_event(
            Event::new("trade")
                .add_attribute("market", "value")
                .add_attribute("base_amount", received_coin.amount.clone())
                .add_attribute("quote_amount", received_coin.amount.clone()),
        )
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin_to_send],
        }))
}

fn default_submit_order_handler() -> StdResult<Response> {
    Ok(Response::new().add_attribute(
        "order_idx",
        rand::thread_rng().gen_range(0..100).to_string(),
    ))
}

fn withdraw_filled_order_handler(
    info: MessageInfo,
    order_ids: Option<Vec<Uint128>>,
) -> StdResult<Response> {
    let mut response = Response::new();
    if let Some(order_ids) = order_ids {
        for _ in order_ids {
            response = response.add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin::new(ONE.into(), DENOM_UTEST.to_string())],
            })
        }
    }
    Ok(response)
}

fn withdraw_partially_filled_order_handler(
    info: MessageInfo,
    order_ids: Option<Vec<Uint128>>,
) -> StdResult<Response> {
    let mut response = Response::new();
    if let Some(order_ids) = order_ids {
        for _ in order_ids {
            response = response.add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![Coin {
                    denom: String::from(DENOM_UTEST),
                    amount: ONE / Uint128::new(2),
                }],
            })
        }
    }
    Ok(response)
}

fn default_retract_order_handler(info: MessageInfo) -> StdResult<Response> {
    Ok(Response::new()
        .add_attribute("amount", (ONE).to_string())
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: String::from(DENOM_UKUJI),
                amount: ONE,
            }],
        }))
}

fn retract_partially_filled_order_handler(info: MessageInfo) -> StdResult<Response> {
    Ok(Response::new()
        .add_attribute("amount", (ONE / Uint128::new(2)).to_string())
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: String::from(DENOM_UKUJI),
                amount: ONE / Uint128::new(2),
            }],
        }))
}

fn default_book_response_handler() -> StdResult<Binary> {
    book_response_handler(
        String::from(DENOM_UTEST),
        Decimal256::from_str("1")?,
        Decimal256::from_str("1")?,
    )
}

fn book_response_handler(
    quote_denom: String,
    base_price: Decimal256,
    quote_price: Decimal256,
) -> StdResult<Binary> {
    let pool_response_quote = PoolResponse {
        quote_price,
        offer_denom: Denom::from(quote_denom.clone()),
        total_offer_amount: Uint256::zero(),
    };

    let pool_response_base = PoolResponse {
        quote_price: base_price,
        offer_denom: Denom::from(quote_denom),
        total_offer_amount: Uint256::zero(),
    };

    to_binary(&BookResponse {
        base: vec![pool_response_base.clone()],
        quote: vec![pool_response_quote.clone()],
    })
}

fn unfilled_order_response(env: Env) -> StdResult<Binary> {
    let response = OrderResponse {
        created_at: env.block.time,
        owner: Addr::unchecked(USER),
        idx: Uint128::new(1),
        quote_price: Decimal256::from_str("1.0").unwrap(),
        original_offer_amount: Uint256::from_str(&ONE.to_string()).unwrap(),
        filled_amount: Uint256::from_str("0").unwrap(),
        offer_denom: Denom::from(DENOM_UKUJI),
        offer_amount: Uint256::from_str(&ONE.to_string()).unwrap(),
    };
    Ok(to_binary(&response)?)
}

fn filled_order_response(env: Env) -> StdResult<Binary> {
    let response = OrderResponse {
        created_at: env.block.time,
        owner: Addr::unchecked(USER),
        idx: Uint128::new(1),
        quote_price: Decimal256::from_str("1.0").unwrap(),
        original_offer_amount: Uint256::from_str(&ONE.to_string()).unwrap(),
        filled_amount: Uint256::from_str(&ONE.to_string()).unwrap(),
        offer_denom: Denom::from(DENOM_UKUJI),
        offer_amount: Uint256::from_str("0").unwrap(),
    };
    Ok(to_binary(&response)?)
}

fn partially_filled_order_response(env: Env) -> StdResult<Binary> {
    let response = OrderResponse {
        idx: Uint128::new(1),
        owner: Addr::unchecked(USER),
        quote_price: Decimal256::from_str("1.0").unwrap(),
        offer_denom: Denom::from(DENOM_UKUJI),
        offer_amount: Uint256::from_str(&ONE.to_string()).unwrap(),
        filled_amount: Uint256::from_str(&(ONE / Uint128::new(2)).to_string()).unwrap(),
        created_at: env.block.time,
        original_offer_amount: Uint256::from_str(&ONE.to_string()).unwrap(),
    };
    Ok(to_binary(&response)?)
}

fn default_query_response() -> StdResult<Binary> {
    #[cw_serde]
    pub struct Mock;
    Ok(to_binary(&Mock)?)
}

pub fn fin_contract_unfilled_limit_order() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |_, _, info, msg: FINExecuteMsg| -> StdResult<Response> {
            match msg {
                FINExecuteMsg::Swap { .. } => default_swap_handler(info),
                FINExecuteMsg::SubmitOrder { .. } => default_submit_order_handler(),
                FINExecuteMsg::WithdrawOrders { order_idxs } => {
                    withdraw_filled_order_handler(info, order_idxs)
                }
                FINExecuteMsg::RetractOrder { .. } => default_retract_order_handler(info),
                _ => Ok(Response::default()),
            }
        },
        |_, _, _, _: FINInstantiateMsg| -> StdResult<Response> { Ok(Response::new()) },
        |_, env, msg: FINQueryMsg| -> StdResult<Binary> {
            match msg {
                FINQueryMsg::Book { .. } => default_book_response_handler(),
                FINQueryMsg::Order { .. } => unfilled_order_response(env),
                _ => default_query_response(),
            }
        },
    );
    Box::new(contract)
}

pub fn fin_contract_partially_filled_order() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |_, _, info, msg: FINExecuteMsg| -> StdResult<Response> {
            match msg {
                FINExecuteMsg::SubmitOrder { .. } => default_submit_order_handler(),
                FINExecuteMsg::RetractOrder { .. } => retract_partially_filled_order_handler(info),
                FINExecuteMsg::WithdrawOrders { order_idxs } => {
                    withdraw_partially_filled_order_handler(info, order_idxs)
                }
                _ => Ok(Response::default()),
            }
        },
        |_, _, _, _: FINInstantiateMsg| -> StdResult<Response> { Ok(Response::new()) },
        |_, env, msg: FINQueryMsg| -> StdResult<Binary> {
            match msg {
                FINQueryMsg::Book { .. } => default_book_response_handler(),
                FINQueryMsg::Order { .. } => partially_filled_order_response(env),
                _ => default_query_response(),
            }
        },
    );
    Box::new(contract)
}

pub fn fin_contract_filled_limit_order() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |_, _, info, msg: FINExecuteMsg| -> StdResult<Response> {
            match msg {
                FINExecuteMsg::Swap { .. } => default_swap_handler(info),
                FINExecuteMsg::SubmitOrder { .. } => default_submit_order_handler(),
                FINExecuteMsg::WithdrawOrders { order_idxs } => {
                    withdraw_filled_order_handler(info, order_idxs)
                }
                FINExecuteMsg::RetractOrder { .. } => default_retract_order_handler(info),
                _ => Ok(Response::default()),
            }
        },
        |_, _, _, _: FINInstantiateMsg| -> StdResult<Response> { Ok(Response::new()) },
        |_, env, msg: FINQueryMsg| -> StdResult<Binary> {
            match msg {
                FINQueryMsg::Book { .. } => default_book_response_handler(),
                FINQueryMsg::Order { .. } => filled_order_response(env),
                _ => default_query_response(),
            }
        },
    );
    Box::new(contract)
}

pub fn fin_contract_pass_slippage_tolerance() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |_, _, info, msg: FINExecuteMsg| -> StdResult<Response> {
            match msg {
                FINExecuteMsg::Swap { .. } => default_swap_handler(info),
                _ => Ok(Response::default()),
            }
        },
        |_, _, _, _: FINInstantiateMsg| -> StdResult<Response> { Ok(Response::new()) },
        |_, _, msg: FINQueryMsg| -> StdResult<Binary> {
            match msg {
                FINQueryMsg::Book { .. } => default_book_response_handler(),
                _ => default_query_response(),
            }
        },
    );
    Box::new(contract)
}

pub fn fin_contract_fail_slippage_tolerance() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        |_, _, _, msg: FINExecuteMsg| -> StdResult<Response> {
            match msg {
                FINExecuteMsg::Swap { .. } => Err(StdError::GenericErr {
                    msg: format!(
                        "Error: code: ({:?}), msg: Max spread exceeded 0.992445703493862134",
                        ERROR_SWAP_SLIPPAGE_EXCEEDED
                    ),
                }),
                _ => Ok(Response::default()),
            }
        },
        |_, _, _, _: FINInstantiateMsg| -> StdResult<Response> { Ok(Response::new()) },
        |_, _, msg: FINQueryMsg| -> StdResult<Binary> {
            match msg {
                FINQueryMsg::Book { .. } => default_book_response_handler(),
                _ => default_query_response(),
            }
        },
    );
    Box::new(contract)
}
