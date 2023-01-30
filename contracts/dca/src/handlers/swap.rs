use crate::{
    error::ContractError,
    state::{
        cache::{SwapCache, SWAP_CACHE},
        pairs::PAIRS,
    },
    types::reply_config::ReplyConfig,
    validation_helpers::{assert_number_of_assets_equals, assert_sender_is_contract},
};
use cosmwasm_std::{Addr, Decimal256, DepsMut, Env, MessageInfo, Response, SubMsg};
use fin_helpers::swaps::create_fin_swap_message;

pub fn swap(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    pair_address: Addr,
    slippage_tolerance: Option<Decimal256>,
    reply_config: Option<ReplyConfig>,
) -> Result<Response, ContractError> {
    assert_sender_is_contract(info, env)?;
    assert_number_of_assets_equals(info.funds.clone(), 1)?;

    let amount = info.funds[0].clone();
    let pair = PAIRS.load(deps.storage, pair_address)?;

    if amount.denom != pair.base_denom && amount.denom != pair.quote_denom {
        return Err(ContractError::CustomError {
            val: format!("Swap denom {} is not in pair {:?}", amount.denom, pair),
        });
    }

    SWAP_CACHE.save(
        deps.storage,
        &SwapCache {
            swap_denom_balance: deps
                .querier
                .query_balance(&env.contract.address, &amount.denom)?,
            receive_denom_balance: deps.querier.query_balance(
                &env.contract.address,
                &if amount.denom == pair.quote_denom {
                    pair.base_denom.clone()
                } else {
                    pair.quote_denom.clone()
                },
            )?,
        },
    )?;

    let swap_message = create_fin_swap_message(deps.querier, pair, amount, slippage_tolerance)?;

    match reply_config {
        Some(reply_config) => {
            return Ok(Response::new().add_submessage(SubMsg {
                id: reply_config.id,
                msg: swap_message,
                gas_limit: None,
                reply_on: reply_config.on,
            }))
        }
        None => Ok(Response::new().add_message(swap_message)),
    }
}
