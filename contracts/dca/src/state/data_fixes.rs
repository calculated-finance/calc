use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_binary, to_binary, Binary, Coin, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, UniqueIndex};

use super::state_helpers::fetch_and_increment_counter;

#[cw_serde]
pub struct DataFix {
    pub id: u64,
    pub resource_id: Uint128,
    pub timestamp: Timestamp,
    pub block_height: u64,
    pub data: DataFixData,
}

#[cw_serde]
pub enum DataFixData {
    VaultAmounts {
        expected_swapped: Coin,
        expected_received: Coin,
    },
    ExecutionCompletedEventAmounts {
        expected_sent: Coin,
        expected_received: Coin,
        expected_fee: Coin,
    },
}

pub struct DataFixBuilder {
    pub resource_id: Uint128,
    pub timestamp: Timestamp,
    pub block_height: u64,
    pub data: DataFixData,
}

impl DataFixBuilder {
    pub fn new(
        resource_id: Uint128,
        timestamp: Timestamp,
        block_height: u64,
        data: DataFixData,
    ) -> DataFixBuilder {
        DataFixBuilder {
            resource_id,
            timestamp,
            block_height,
            data,
        }
    }

    pub fn build(self, id: u64) -> DataFix {
        DataFix {
            id,
            resource_id: self.resource_id,
            timestamp: self.timestamp,
            block_height: self.block_height,
            data: self.data,
        }
    }
}

const DATA_FIX_COUNTER: Item<u64> = Item::new("data_fix_counter_v20");

pub struct DataFixIndexes<'a> {
    pub resource_id: UniqueIndex<'a, (u128, u64), Binary, u64>,
}

impl<'a> IndexList<Binary> for DataFixIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Binary>> + '_> {
        let v: Vec<&dyn Index<Binary>> = vec![&self.resource_id];
        Box::new(v.into_iter())
    }
}

pub fn data_fix_store<'a>() -> IndexedMap<'a, u64, Binary, DataFixIndexes<'a>> {
    let indexes = DataFixIndexes {
        resource_id: UniqueIndex::new(
            |data_fix| {
                from_binary(&data_fix)
                    .map(|data_fix: DataFix| (data_fix.resource_id.into(), data_fix.id))
                    .expect("data_fix")
            },
            "data_fixes_v20__resource_id",
        ),
    };
    IndexedMap::new("data_fixes_v20", indexes)
}

pub fn save_data_fix(store: &mut dyn Storage, data_fix_builder: DataFixBuilder) -> StdResult<()> {
    let data_fix_id = fetch_and_increment_counter(store, DATA_FIX_COUNTER)?;
    let data_fix = data_fix_builder.build(data_fix_id);
    data_fix_store().save(store, data_fix_id.clone(), &to_binary(&data_fix)?)
}
