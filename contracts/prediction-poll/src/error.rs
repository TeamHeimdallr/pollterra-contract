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

    #[error("already participated")]
    AlreadyParticipated {},

    #[error("you need to send some ust in order to bet")]
    EmptyFunds {},

    #[error("you'd better not send funds")]
    NotEmptyFunds {},

    #[error("already finished poll")]
    AlreadyFinishedPoll {},

    #[error("Vote is live now, The poll cannot be finished before the end time")]
    FinishBeforeEndTime {},

    #[error("Already reclaimed")]
    AlreadyReclaimed {},

    #[error("The bet amount should be over {0}")]
    LessThanMinimumAmount(Uint128),

    #[error("only send ust to bet")]
    OnlyUstAvailable {},

    #[error("already reverted poll")]
    AlreadyReverted {},

    #[error("there's no rewards to claim")]
    EmptyRewards {},

    #[error("You can't reset the poll until the poll is closed")]
    ResetBeforeClosed {},

    #[error("cannot claim rewrads, current status: {0}")]
    CannotClaimRewards(PollStatus),

    #[error("Not enough total amount, {0} is less than {1}")]
    NotEnoughTotalAmount(Uint128, Uint128),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
