use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
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

pub const BETS: Map<(&[u8], &Addr), Uint128> = Map::new("bets"); // (side, addr): amount
pub const USER_TOTAL_AMOUNT: Map<&Addr, Uint128> = Map::new("user_total_amount"); // addr: amount
pub const SIDE_TOTAL_AMOUNT: Map<&[u8], Uint128> = Map::new("side_total_amount"); // side: amount
pub const REWARDS: Map<&Addr, Uint128> = Map::new("rewards"); // addr: amount
pub const STATE: Item<State> = Item::new("state");

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}
