use crate::{
    state::config::{update_config, Config},
    validation::assert_sender_is_admin,
};
use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

pub fn update_config_handler(
    deps: DepsMut,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    assert_sender_is_admin(deps.storage, info.sender)?;
    deps.api.addr_validate(&config.admin.to_string())?;
    update_config(deps.storage, config)?;
    Ok(Response::new().add_attribute("method", "update_config"))
}
