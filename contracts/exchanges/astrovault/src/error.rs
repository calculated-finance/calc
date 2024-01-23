use cosmwasm_std::{StdError, OverflowError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Failed swap: {msg:?}")]
    FailedSwap { msg: String },

    #[error("Missing reply id")]
    MissingReplyId {},

    #[error("Empty fin pool")]
    EmptyPool {},

    #[error("Invalid pair info: {msg:?}")]
    InvalidPair { msg: String },

    #[error("Assets cannot be the same")]
    SameAsset {},

    #[error("Pair already exists")]
    PairExist {},

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),
}
