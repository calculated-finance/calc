use crate::helpers::price::FinSimulationResponse;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Binary, ContractResult, CustomQuery, Empty, OwnedDeps,
    Querier, QuerierResult, QueryRequest, StdError, StdResult, SystemError, SystemResult,
    WasmQuery,
};
use kujira::fin::QueryMsg as FinQueryMsg;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

pub const USER: &str = "user";
pub const ADMIN: &str = "admin";
pub const FEE_COLLECTOR: &str = "fee_collector";
pub const VALIDATOR: &str = "validator";

pub const DENOM_UDEMO: &str = "udemo";
pub const DENOM_UKUJI: &str = "ukuji";
pub const DENOM_UATOM: &str = "uatom";
pub const DENOM_UION: &str = "uion";
pub const DENOM_USDC: &str = "uaxlusdc";

pub struct CalcMockQuerier<C: DeserializeOwned = Empty> {
    default_stargate_handler: Box<dyn for<'a> Fn(&'a str, &Binary) -> StdResult<Binary>>,
    stargate_handler: Box<dyn for<'a> Fn(&'a str, &Binary) -> StdResult<Binary>>,
    mock_querier: MockQuerier<C>,
}

impl<C: DeserializeOwned> CalcMockQuerier<C> {
    pub fn new() -> Self {
        Self {
            default_stargate_handler: Box::new(|path, __| panic!("Unexpected path: {}", path)),
            stargate_handler: Box::new(|_, __| {
                Err(StdError::generic_err(
                    "no custom stargate handler, should invoke the default handler",
                ))
            }),
            mock_querier: MockQuerier::<C>::new(&[]),
        }
    }
}

impl<C: CustomQuery + DeserializeOwned> Querier for CalcMockQuerier<C> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<C> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl<C: CustomQuery + DeserializeOwned> CalcMockQuerier<C> {
    pub fn update_stargate<WH: 'static>(&mut self, stargate_handler: WH)
    where
        WH: Fn(&str, &Binary) -> StdResult<Binary>,
    {
        self.stargate_handler = Box::from(stargate_handler);
    }

    pub fn update_wasm<WH: 'static>(&mut self, wasm_handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult,
    {
        self.mock_querier.update_wasm(wasm_handler);
    }

    pub fn handle_query(&self, request: &QueryRequest<C>) -> QuerierResult {
        match &request {
            QueryRequest::Stargate { path, data } => SystemResult::Ok(ContractResult::Ok(
                (*self.stargate_handler)(path, data)
                    .unwrap_or_else(|_| (*self.default_stargate_handler)(path, data).unwrap()),
            )),
            _ => self.mock_querier.handle_query(request),
        }
    }
}

pub fn calc_mock_dependencies() -> OwnedDeps<MockStorage, MockApi, CalcMockQuerier, Empty> {
    let mut deps = OwnedDeps {
        storage: MockStorage::new(),
        api: MockApi::default(),
        querier: CalcMockQuerier::new(),
        custom_query_type: PhantomData,
    };

    deps.querier.update_wasm(|query| {
        SystemResult::Ok(ContractResult::Ok(match query {
            WasmQuery::Smart { msg, .. } => match from_binary(&msg).unwrap() {
                FinQueryMsg::Simulation { offer_asset } => to_binary(&FinSimulationResponse {
                    return_amount: offer_asset.amount,
                })
                .unwrap(),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }))
    });

    deps
}
