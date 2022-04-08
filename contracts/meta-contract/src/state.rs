use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub admins: Option<Vec<Addr>>,
    pub token_contract: String,
    pub creation_deposit: Uint128,
    pub reclaimable_threshold: Uint128,
    pub minimum_bet_amount: Uint128,
    pub tax_percentage: Decimal,
    // TODO : participation requirement of opinion poll
}

impl Config {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        CONFIG.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Config> {
        CONFIG.load(storage)
    }

    pub fn is_admin(&self, address: &Addr) -> bool {
        let admin_check = match self.admins.as_ref() {
            Some(admin_list) => admin_list.contains(address),
            None => false,
        };
        admin_check || self.owner == *address
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub num_contract: u64,
}

impl State {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<State> {
        STATE.load(storage)
    }
}

pub const STATE: Item<State> = Item::new("state");
pub const CONFIG: Item<Config> = Item::new("config");
pub const CONTRACTS: Map<&Addr, ()> = Map::new("contracts");
