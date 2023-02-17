use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Map;

pub const ALLIANCES: Map<String, bool> = Map::new("alliances_v1");

pub fn create_alliance(storage: &mut dyn Storage, alliance: String) -> StdResult<()> {
    ALLIANCES.save(storage, alliance, &true)
}

pub fn delete_alliance(storage: &mut dyn Storage, alliance: String) -> StdResult<()> {
    ALLIANCES.remove(storage, alliance);
    Ok(())
}

pub fn is_alliance(storage: &dyn Storage, alliance: String) -> StdResult<bool> {
    Ok(ALLIANCES.may_load(storage, alliance)?.unwrap_or(false))
}
