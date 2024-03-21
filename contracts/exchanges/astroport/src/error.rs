use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Must provide a route")]
    Route {},

    #[error("Must provide a single asset for swapping")]
    AssetCardinality {},

    #[error("Must provide a non zero swap amount")]
    SwapAmount {},

    #[error("Receive amount was less than the minimum specified")]
    ReceiveAmount {},

    #[error("Missing reply id")]
    MissingReplyId {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> StdError {
        StdError::GenericErr {
            msg: err.to_string(),
        }
    }
}
