use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PredictionPollConfig {
    // TODO : Have too many fields. Need to split it.
    pub owner: Addr,
    pub generator: Addr,
    pub token_contract: String,
    pub deposit_amount: Uint128,
    pub deposit_reclaimed: bool,
    pub reclaimable_threshold: Uint128,
    pub status: PollStatus,
    pub poll_name: String,
    pub bet_end_time: u64,
    pub total_amount: Uint128,
    pub minimum_bet: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PollStatus {
    Voting,
    Reward,
    Closed,
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PollStatus::Voting => write!(f, "Voting"),
            PollStatus::Reward => write!(f, "Reward"),
            PollStatus::Closed => write!(f, "Closed"),
        }
    }
}
