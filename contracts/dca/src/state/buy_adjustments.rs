use cosmwasm_std::{Decimal, StdResult, Storage};
use cw_storage_plus::Map;

const BUY_ADJUSTMENTS: Map<u8, Decimal> = Map::new("buy_adjustments_v20");

pub fn save_buy_adjustments(
    storage: &mut dyn Storage,
    adjustments: Vec<(u8, Decimal)>,
) -> StdResult<()> {
    for (model, adjustment) in adjustments {
        BUY_ADJUSTMENTS.save(storage, model, &adjustment)?;
    }
    Ok(())
}

pub fn get_buy_adjustment(storage: &dyn Storage, model: u8) -> StdResult<Decimal> {
    BUY_ADJUSTMENTS.load(storage, model)
}
