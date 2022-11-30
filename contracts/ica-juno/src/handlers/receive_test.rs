use base::ibc::msg::{KCalc};
use cosmwasm_std::{DepsMut, IbcReceiveResponse, StdResult};

use crate::{
    ibc_msg::StdAck,
    state::{State, STATE},
};

pub fn receive_test(
    deps: DepsMut,
    caller: String,
    value: String,
) -> StdResult<IbcReceiveResponse> {
    // save some state to this contract so we can query and see
    let state = State {
        value: value.clone(),
        caller: caller.clone(),
    };
    STATE.save(deps.storage, &state)?;

    // send response message
    let res = KCalc::TestResponse {
        value: format!("host chain receieved: {} from {} yolo", value, caller),
    };

    let acknowledgement = StdAck::success(&res);

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("method", "receive_test"))
}
