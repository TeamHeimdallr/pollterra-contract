use cosmwasm_std::{Addr, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollConfig {
    pub owner: Addr,
    pub generator: Addr,
    pub token_contract: String,
    pub reclaimable_threshold: Uint128,
    pub poll_name: String,
    pub poll_type: PollType,
    pub end_time: u64,
    // only for prediction poll
    pub resolution_time: u64,
    pub minimum_bet_amount: Uint128,
    pub tax_percentage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollState {
    pub deposit_amount: Uint128,
    pub deposit_reclaimed: bool,
    pub status: PollStatus,
    pub total_amount: Uint128,
    pub winning_side: Option<Vec<u64>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PollStatus {
    Voting,
    Reward,
    Closed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PollType {
    Prediction,
    Opinion,
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
