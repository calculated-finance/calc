use cosmwasm_std::{Decimal, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};
use fin_helpers::position_type::PositionType;

const BUY_ADJUSTMENTS: Map<u8, Decimal> = Map::new("buy_adjustments_v20");
const SELL_ADJUSTMENTS: Map<u8, Decimal> = Map::new("sell_adjustments_v20");
const BUY_ADJUSTMENTS_UPDATED_AT: Item<Timestamp> = Item::new("buy_adjustments_updated_at_v20");
const SELL_ADJUSTMENTS_UPDATED_AT: Item<Timestamp> = Item::new("buy_adjustments_updated_at_v20");

fn last_updated(storage: &dyn Storage, position_type: PositionType) -> StdResult<Timestamp> {
    match position_type {
        PositionType::Enter => BUY_ADJUSTMENTS_UPDATED_AT.load(storage),
        PositionType::Exit => SELL_ADJUSTMENTS_UPDATED_AT.load(storage),
    }
}

fn adjustments_updated_store(position_type: PositionType) -> &'static Item<'static, Timestamp> {
    match position_type {
        PositionType::Enter => &BUY_ADJUSTMENTS_UPDATED_AT,
        PositionType::Exit => &SELL_ADJUSTMENTS_UPDATED_AT,
    }
}

pub fn adjustments_store(position_type: PositionType) -> &'static Map<'static, u8, Decimal> {
    match position_type {
        PositionType::Enter => &BUY_ADJUSTMENTS,
        PositionType::Exit => &SELL_ADJUSTMENTS,
    }
}

pub fn update_swap_adjustments(
    storage: &mut dyn Storage,
    position_type: PositionType,
    adjustments: Vec<(u8, Decimal)>,
    block_time: Timestamp,
) -> StdResult<()> {
    for (model, adjustment) in adjustments {
        adjustments_store(position_type.clone()).save(storage, model, &adjustment)?;
    }
    adjustments_updated_store(position_type).save(storage, &block_time)
}

pub fn get_swap_adjustment(
    storage: &dyn Storage,
    position_type: PositionType,
    model: u8,
    block_time: Timestamp,
) -> StdResult<Decimal> {
    let last_updated = last_updated(storage, position_type.clone())?;
    if last_updated < block_time.minus_seconds(60 * 60 * 30) {
        return Ok(Decimal::one());
    }
    adjustments_store(position_type).load(storage, model)
}
