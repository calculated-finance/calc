use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
    msg::MigrateMsg,
};
use cosmwasm_std::{DepsMut, Env, Response};
use cw2::set_contract_version;

pub fn migrate_handler(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("migrate", "true")
        .add_attribute("msg", format!("{:#?}", msg)))
}
