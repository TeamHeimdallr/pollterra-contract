use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub token_contract: Addr,
    pub creation_deposit: Uint128,
    pub num_contract: u64,
}

pub const STATE: Item<State> = Item::new("state");
pub const CONTRACTS: Map<&Addr, ()> = Map::new("contracts");

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    InitPoll {
        code_id: u64,
        poll_name: String,
        start_time: u64,
        bet_end_time: u64,
    },
}
