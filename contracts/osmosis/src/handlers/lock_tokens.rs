use crate::{cache::CACHE, ContractError};
use cosmwasm_std::{DepsMut, Env, Response, SubMsg};
use osmosis_std::{shim::Duration, types::osmosis::lockup::MsgLockTokens};

pub fn lock_tokens(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let cache = CACHE.load(deps.storage)?;

    let balance = deps.querier.query_balance(
        env.contract.address.clone(),
        format!("gamm/pool/{}", cache.pool_id),
    )?;

    let seconds_in_24_hours = 86400;

    let lock = MsgLockTokens {
        owner: env.contract.address.into(),
        duration: Some(Duration {
            seconds: seconds_in_24_hours,
            nanos: 0,
        }),
        coins: vec![balance.into()],
    };

    Ok(Response::new()
        .add_attribute("method", "lock_tokens")
        .add_submessage(SubMsg::new(lock)))
}
