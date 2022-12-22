use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Response, Uint128, Uint64};

use crate::error::ContractError;

pub fn fix_event_amounts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: Uint128,
    event_id: Uint64,
    expected_sent: Coin,
    expected_received: Coin,
    expected_fee: Coin,
) -> Result<Response, ContractError> {
    unimplemented!()
}
