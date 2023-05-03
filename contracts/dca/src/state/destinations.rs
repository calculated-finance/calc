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

pub const DESTINATIONS: Map<u128, Binary> = Map::new("destinations_v20");

pub fn get_destinations(store: &dyn Storage, vault_id: Uint128) -> StdResult<Vec<OldDestination>> {
    let destinations = DESTINATIONS.may_load(store, vault_id.into())?;
    match destinations {
        Some(destinations) => Ok(from_binary(&destinations)?),
        None => Ok(vec![]),
    }
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
