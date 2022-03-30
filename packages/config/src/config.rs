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
    pub reclaimed: bool,
    pub reclaimable_threshold: Uint128,
    pub status: BetStatus,
    pub bet_live: bool,
    pub reward_live: bool,
    pub poll_name: String,
    pub start_time: u64,
    pub bet_end_time: u64,
    pub total_amount: Uint128,
    pub minimum_bet: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum BetStatus {
    Created,
    Betting,
    BetHold,
    Reward,
    Closed,
}

impl fmt::Display for BetStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BetStatus::Created => write!(f, "Created"),
            BetStatus::Betting => write!(f, "Betting"),
            BetStatus::BetHold => write!(f, "BetHold"),
            BetStatus::Reward => write!(f, "Reward"),
            BetStatus::Closed => write!(f, "Closed"),
        }
    }
}
