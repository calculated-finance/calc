use cosmwasm_std::Deps;
use cosmwasm_std::StdResult;

use super::get_fund::get_fund;
use fund_core::msg::ConfigResponse as FundConfigResponse;
use fund_core::msg::QueryMsg as FundQueryMsg;

pub fn get_fund_config(deps: Deps) -> StdResult<FundConfigResponse> {
    let fund = get_fund(deps)?
        .address
        .expect("config should be set after fund is assigned");

    deps.querier
        .query_wasm_smart(fund.clone(), &FundQueryMsg::GetConfig {})
}
