use config::config::{PollConfig, PollState, PollStatus};
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};

pub type Config = PollConfig;
pub type State = PollState;

pub type BetStatus = PollStatus;

pub const VOTES: Map<&Addr, u64> = Map::new("votes"); // addr: side
pub const SIDES: Map<&[u8], u64> = Map::new("sides"); // side(u64): count
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
