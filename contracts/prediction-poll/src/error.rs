use config::config::PollStatus;
use cosmwasm_std::{StdError, Timestamp, Uint128};
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Bet is not live. current block time: {0}, bet end time: {1}")]
    BetIsNotLive(Timestamp, u64),

    #[error("You need to send some ust in order to bet")]
    EmptyFunds {},

    #[error("You'd better not send funds")]
    NotEmptyFunds {},

    #[error("Already finished poll")]
    AlreadyFinishedPoll {},

    #[error("Vote is live now, The poll cannot be finished before the end time")]
    FinishBeforeEndTime {},

    #[error("Already reclaimed")]
    AlreadyReclaimed {},

    #[error("The bet amount should be over {0}")]
    LessThanMinimumBetAmount(Uint128),

    #[error("Only send ust to bet")]
    OnlyUstAvailable {},

    #[error("Already reverted poll")]
    AlreadyReverted {},

    #[error("There's no rewards to claim")]
    EmptyRewards {},

    #[error("You can't reset the poll until the poll is closed")]
    ResetBeforeClosed {},

    #[error("Cannot claim rewards, current status: {0}")]
    CannotClaimRewards(PollStatus),

    #[error("Not enough total amount to reclaim the deposit, {0} is less than {1}")]
    InsufficientReclaimableThreshold(Uint128, Uint128),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
