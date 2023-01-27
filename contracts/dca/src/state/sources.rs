use crate::types::source::Source;
use cosmwasm_std::{from_binary, to_binary, Binary, StdResult, Storage, Uint128};
use cw_storage_plus::Map;

const SOURCES: Map<u128, Binary> = Map::new("sources_v20");

pub fn save_source(store: &mut dyn Storage, vault_id: Uint128, source: Source) -> StdResult<()> {
    SOURCES.save(store, vault_id.into(), &to_binary(&source)?)
}

pub fn get_source(store: &dyn Storage, vault_id: Uint128) -> StdResult<Option<Source>> {
    let source = SOURCES.may_load(store, vault_id.into())?;
    match source {
        Some(source) => Ok(Some(from_binary(&source)?)),
        None => Ok(None),
    }
}

pub fn update_source(store: &mut dyn Storage, vault_id: Uint128, source: Source) -> StdResult<()> {
    SOURCES.save(store, vault_id.into(), &to_binary(&source)?)
}

pub fn remove_source(store: &mut dyn Storage, vault_id: Uint128) -> StdResult<()> {
    Ok(SOURCES.remove(store, vault_id.into()))
}
