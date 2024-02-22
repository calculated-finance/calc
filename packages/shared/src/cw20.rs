use cosmwasm_std::{Coin, Deps, MessageInfo, StdResult};
use cw20::Cw20ReceiveMsg;

pub fn from_cw20(
    deps: &Deps,
    info: MessageInfo,
    receive_msg: Cw20ReceiveMsg,
) -> StdResult<MessageInfo> {
    let sender = deps.api.addr_validate(receive_msg.sender.as_ref())?;
    let contract_address = deps.api.addr_validate(info.sender.as_ref())?;

    Ok(MessageInfo {
        sender,
        funds: vec![Coin {
            amount: receive_msg.amount,
            denom: contract_address.to_string(),
        }],
    })
}
