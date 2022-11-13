use base::events::event::{Event, EventBuilder};
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, UniqueIndex};

use super::state_helpers::fetch_and_increment_counter;

const EVENT_COUNTER: Item<u64> = Item::new("event_counter_v11");

pub struct EventIndexes<'a> {
    pub resource_id: UniqueIndex<'a, (u128, u64), Event, u64>,
}

impl<'a> IndexList<Event> for EventIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Event>> + '_> {
        let v: Vec<&dyn Index<Event>> = vec![&self.resource_id];
        Box::new(v.into_iter())
    }
}

pub fn event_store<'a>() -> IndexedMap<'a, u64, Event, EventIndexes<'a>> {
    let indexes = EventIndexes {
        resource_id: UniqueIndex::new(|e| (e.resource_id.into(), e.id), "events_v11__resource_id"),
    };
    IndexedMap::new("events_v11", indexes)
}

pub fn create_event(store: &mut dyn Storage, event_builder: EventBuilder) -> StdResult<u64> {
    let event = event_builder.build(fetch_and_increment_counter(store, EVENT_COUNTER)?.into());
    event_store().save(store, event.id, &event.clone())?;
    Ok(event.id)
}

pub fn create_events(store: &mut dyn Storage, event_builders: Vec<EventBuilder>) -> StdResult<()> {
    for event_builder in event_builders {
        create_event(store, event_builder)?;
    }
    Ok(())
}

pub fn clear_events(store: &mut dyn Storage) {
    event_store().clear(store);
    EVENT_COUNTER.remove(store)
}
