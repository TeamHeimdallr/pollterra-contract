use cosmwasm_std::{StdError, Timestamp, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Vote is not live. current block time: {0}, vote end time: {1}")]
    VoteIsNotLive(Timestamp, u64),

    #[error("already participated")]
    AlreadyParticipated {},

    #[error("you'd better not send funds")]
    NotEmptyFunds {},

    #[error("already finished poll")]
    AlreadyFinishedPoll {},

    #[error("Vote is live now, The poll cannot be finished before the end time")]
    FinishBeforeEndTime {},

    #[error("Already reclaimed")]
    AlreadyReclaimed {},

    #[error("Not enough total amount, {0} is less than {1}")]
    NotEnoughTotalAmount(Uint128, Uint128),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
