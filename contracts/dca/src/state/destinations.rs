use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_binary, to_binary, Addr, Binary, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::Map;

use crate::{msg::ExecuteMsg, types::destination::Destination};

#[cw_serde]
pub enum PostExecutionAction {
    Send,
    ZDelegate,
    Custom {
        contract_addr: Addr,
        msg: Option<Binary>,
    },
}

#[cw_serde]
pub struct OldDestination {
    pub address: Addr,
    pub allocation: Decimal,
    pub action: PostExecutionAction,
}

const DESTINATIONS: Map<u128, Binary> = Map::new("destinations_v20");

pub fn get_destinations(
    store: &dyn Storage,
    vault_id: Uint128,
    owner: Addr,
    contract_address: Addr,
) -> StdResult<Vec<Destination>> {
    let old_destinations = DESTINATIONS.may_load(store, vault_id.into())?;

    Ok(if let Some(old_destinations) = old_destinations {
        from_binary::<Vec<OldDestination>>(&old_destinations)?
            .iter()
            .map(|d| destination_from(d, owner.clone(), contract_address.clone()))
            .collect::<Vec<Destination>>()
    } else {
        vec![]
    })
}

pub fn save_destinations(
    store: &mut dyn Storage,
    vault_id: Uint128,
    destinations: &Vec<Destination>,
) -> StdResult<()> {
    let old_destinations = destinations
        .into_iter()
        .map(|d| old_destination_from(d.clone()))
        .collect::<Vec<OldDestination>>();
    DESTINATIONS.save(store, vault_id.into(), &to_binary(&old_destinations)?)?;
    Ok(())
}

pub fn destination_from(
    old_destination: &OldDestination,
    owner: Addr,
    contract_address: Addr,
) -> Destination {
    match old_destination.action.clone() {
        PostExecutionAction::Send => Destination {
            address: old_destination.address.clone(),
            allocation: old_destination.allocation,
            msg: None,
        },
        PostExecutionAction::ZDelegate => Destination {
            address: contract_address,
            allocation: old_destination.allocation,
            msg: Some(
                to_binary(&ExecuteMsg::ZDelegate {
                    delegator_address: owner,
                    validator_address: old_destination.address.clone(),
                })
                .unwrap(),
            ),
        },
        PostExecutionAction::Custom { contract_addr, msg } => Destination {
            address: contract_addr,
            allocation: old_destination.allocation,
            msg,
        },
    }
}

pub fn old_destination_from(destination: Destination) -> OldDestination {
    if let Some(msg) = destination.msg.clone() {
        from_binary::<ExecuteMsg>(&msg).map_or_else(
            |_| OldDestination {
                address: destination.address.clone(),
                allocation: destination.allocation.clone(),
                action: PostExecutionAction::Custom {
                    contract_addr: destination.address.clone(),
                    msg: destination.msg.clone(),
                },
            },
            |execute_msg| match execute_msg.clone() {
                ExecuteMsg::ZDelegate {
                    validator_address, ..
                } => OldDestination {
                    address: validator_address,
                    allocation: destination.allocation,
                    action: PostExecutionAction::ZDelegate,
                },
                _ => OldDestination {
                    address: destination.address.clone(),
                    allocation: destination.allocation,
                    action: PostExecutionAction::Custom {
                        contract_addr: destination.address.clone(),
                        msg: destination.msg.clone(),
                    },
                },
            },
        )
    } else {
        OldDestination {
            address: destination.address,
            allocation: destination.allocation,
            action: PostExecutionAction::Send,
        }
    }
}
