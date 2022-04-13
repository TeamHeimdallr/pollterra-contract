use crate::state::{BetStatus, Config, State};
use cosmwasm_std::Uint128;
use messages::msg::PollInstantiateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

pub type InstantiateMsg = PollInstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Bet { side: u64 },
    FinishPoll { winner: u64 },
    RevertPoll {},
    Claim {},
    ReclaimDeposit {},
    TransferOwner { new_owner: String },
    SetMinimumBet { amount: u128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    BetLive {},
    RewardLive {},
    UserBet { address: String, side: u64 },
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

pub type ConfigResponse = Config;
pub type StateResponse = State;
