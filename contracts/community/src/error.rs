use cosmwasm_std::{OverflowError, StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized: only admins are able")]
    Unauthorized {},

    #[error("Invalid zero amount: do not send or transfer zero value")]
    InvalidZeroAmount {},

    #[error("Insufficient free balance. Current free balance is {0}")]
    InsufficientFreeBalance(Uint128),

    #[error("Insufficient remain amount of the address. Current remain amount is {0}")]
    InsufficientRemainAmount(Uint128),
}
