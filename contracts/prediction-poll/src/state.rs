use config::config::{PredictionPollConfig, PredictionPollStatus};
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

pub type State = PredictionPollConfig;

pub type BetStatus = PredictionPollStatus;

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
