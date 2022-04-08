use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply id")]
    InvalidReplyId {},

    #[error("Incorrect token contract")]
    IncorrectTokenContract {},

    #[error("Must deposit more than {0} token")]
    InsufficientTokenDeposit(Uint128),

    #[error("Deposit doesn't match, should be {0} token")]
    InvalidTokenDeposit(Uint128),

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Cw20Msg doesn't match")]
    InvalidCw20Msg {},

    #[error("Token contract is already registered")]
    TokenAlreadyRegistered {},

    #[error("Empty winner")]
    EmptyWinner {},

    #[error("Token contract is not registered")]
    TokenNotRegistered {},

    #[error("Poll type should be one of (prediction | opinion)")]
    InvalidPollType {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
