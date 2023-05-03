use crate::{
    error::ContractError,
    msg::MigrateMsg,
    state::{old_pairs::PAIRS, old_triggers::TRIGGERS, pairs::save_pair, triggers::save_trigger},
    types::{pair::Pair, trigger::Trigger},
};
use cosmwasm_std::{DepsMut, Order, Response};

pub fn migrate_handler(deps: DepsMut, msg: MigrateMsg) -> Result<Response, ContractError> {
    let pairs = PAIRS
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|result| result.map(|(_, pair)| pair))
        .collect::<Vec<Pair>>();

    for pair in pairs {
        save_pair(deps.storage, &pair)?;
    }

    let triggers = TRIGGERS
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|result| result.map(|(_, trigger)| trigger))
        .collect::<Vec<Trigger>>();

    for trigger in triggers {
        save_trigger(deps.storage, trigger)?;
    }

    Ok(Response::new()
        .add_attribute("method", "migrate")
        .add_attribute("msg", format!("{:#?}", msg)))
}
