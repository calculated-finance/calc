use crate::{
    constants::TWO_MICRONS,
    contract::{
        AFTER_FIN_LIMIT_ORDER_CREATED_FOR_MIGRATE_REPLY_ID,
        AFTER_FIN_LIMIT_ORDER_RETRACTED_FOR_MIGRATE_REPLY_ID,
    },
    error::ContractError,
    state::vaults::get_vault,
};
use base::triggers::trigger::TriggerConfiguration;
use cosmwasm_std::{Coin, DepsMut, Response, Uint128};
use fin_helpers::{
    limit_orders::{create_limit_order_sub_msg, create_retract_order_sub_msg},
    queries::query_order_details,
};

pub fn migrate_fin_limit_order(
    deps: DepsMut,
    vault_id: Uint128,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    let vault = get_vault(deps.storage, vault_id)?;

    match vault.trigger.as_ref().expect("fin limit order trigger") {
        TriggerConfiguration::FinLimitOrder { order_idx, .. } => match order_idx {
            Some(order_idx) => {
                let limit_order_details = query_order_details(
                    deps.querier,
                    vault.pair.address.clone(),
                    order_idx.to_owned(),
                )?;

                response = response.add_submessage(create_retract_order_sub_msg(
                    vault.pair.address.clone(),
                    order_idx.to_owned(),
                    AFTER_FIN_LIMIT_ORDER_RETRACTED_FOR_MIGRATE_REPLY_ID,
                ));

                Ok(response.add_submessage(create_limit_order_sub_msg(
                    vault.pair.address.clone(),
                    limit_order_details.quote_price,
                    Coin::new(TWO_MICRONS.into(), vault.get_swap_denom()),
                    AFTER_FIN_LIMIT_ORDER_CREATED_FOR_MIGRATE_REPLY_ID,
                )))
            }
            _ => panic!("vault does not have a fin limit trigger"),
        },
        _ => panic!("vault does not have a fin limit trigger"),
    }
}
