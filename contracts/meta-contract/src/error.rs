use cosmwasm_std::StdError;
use cosmwasm_std::Uint128;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply id")]
    InvalidReplyId {},

    #[error("incorrect token contract")]
    IncorrectTokenContract {},

    #[error("Must deposit more than {0} token")]
    InsufficientTokenDeposit(Uint128),

    #[error("Inssuficient Balance")]
    InssuficientBalance {},

    #[error("Cw20Msg doesn't match")]
    InvalidCw20Msg {},

    #[error("token contract is already registered")]
    TokenAlreadyRegistered {},

    #[error("token contract is not registered")]
    TokenNotRegistered {},

    #[error("poll type should be one of (prediction | opinion)")]
    InvalidPollType {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
