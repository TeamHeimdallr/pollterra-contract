use crate::state::{BetStatus, State};
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub poll_name: String,
    pub start_time: u64,
    pub bet_end_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Bet {
        side: u8,
    },
    FinishPoll {
        winner: u8,
    },
    RevertPoll {},
    Claim {},
    ResetPoll {
        poll_name: String,
        start_time: u64,
        bet_end_time: u64,
    },
    TransferOwner {
        new_owner: String,
    },
    SetMinimumBet {
        amount: u128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BetLive {},
    RewardLive {},
    UserBet { address: String, side: u8 },
    UserRewards { address: String },
    BetStatus {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BetLiveResponse {
    pub bet_live: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardLiveResponse {
    pub reward_live: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BetStatusResponse {
    pub status: BetStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserBetResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserRewardsResponse {
    pub reward: Uint128,
}

pub type ConfigResponse = State;
