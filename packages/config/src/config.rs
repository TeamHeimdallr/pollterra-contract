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
    pub status: PredictionPollStatus,
    pub bet_live: bool,
    pub reward_live: bool,
    pub poll_name: String,
    pub start_time: u64,
    pub bet_end_time: u64,
    pub total_amount: Uint128,
    pub minimum_bet: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PredictionPollStatus {
    Created,
    Betting,
    BetHold,
    Reward,
    Closed,
}

impl fmt::Display for PredictionPollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PredictionPollStatus::Created => write!(f, "Created"),
            PredictionPollStatus::Betting => write!(f, "Betting"),
            PredictionPollStatus::BetHold => write!(f, "BetHold"),
            PredictionPollStatus::Reward => write!(f, "Reward"),
            PredictionPollStatus::Closed => write!(f, "Closed"),
        }
    }
}
