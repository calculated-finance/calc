use super::{
    continue_swap::continue_swap_handler, send_funds::send_funds_handler,
    swap_on_fin::swap_on_fin_handler,
};
use crate::{
    contract::ContractResult, msg::ExecuteInternalMsg, validation::assert_sender_is_contract,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn execute_internal_message_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    message: ExecuteInternalMsg,
) -> ContractResult<Response> {
    assert_sender_is_contract(&info.sender, &env)?;

    match message {
        ExecuteInternalMsg::ContinueSwap { swap_id } => continue_swap_handler(deps, info, swap_id),
        ExecuteInternalMsg::SwapOnFin {
            pair,
            slippage_tolerance,
            callback,
        } => swap_on_fin_handler(deps, &env, &info, pair, slippage_tolerance, callback),
        ExecuteInternalMsg::SendFunds { address } => send_funds_handler(info, address),
    }
}
