use crate::{
    error::ContractError,
    state::config::{get_config, update_config, Config},
    validation_helpers::assert_sender_is_admin,
};
use cosmwasm_std::{Addr, Decimal, DepsMut, MessageInfo, Response};

pub fn update_config_handler(
    deps: DepsMut,
    info: MessageInfo,
    fee_collector: Option<Addr>,
    fee_percent: Option<Decimal>,
    automation_fee_percent: Option<Decimal>,
    staking_router_address: Option<Addr>,
    page_limit: Option<u16>,
    paused: Option<bool>,
) -> Result<Response, ContractError> {
    assert_sender_is_admin(deps.storage, info.sender)?;
    let existing_config = get_config(deps.storage)?;

    let config = update_config(
        deps.storage,
        Config {
            admin: existing_config.admin,
            fee_collector: deps.api.addr_validate(
                &fee_collector
                    .unwrap_or(existing_config.fee_collector)
                    .to_string(),
            )?,
            fee_percent: fee_percent.unwrap_or(existing_config.fee_percent),
            automation_fee_percent: automation_fee_percent
                .unwrap_or(existing_config.automation_fee_percent),
            staking_router_address: deps.api.addr_validate(
                &staking_router_address
                    .unwrap_or(existing_config.staking_router_address)
                    .to_string(),
            )?,
            page_limit: page_limit.unwrap_or(existing_config.page_limit),
            paused: paused.unwrap_or(existing_config.paused),
        },
    )?;

    Ok(Response::default()
        .add_attribute("method", "update_config")
        .add_attribute("fee_percent", config.fee_percent.to_string())
        .add_attribute("fee_collector", config.fee_collector.to_string())
        .add_attribute(
            "automation_fee_percent",
            config.automation_fee_percent.to_string(),
        )
        .add_attribute(
            "staking_router_address",
            config.staking_router_address.to_string(),
        ))
}
