use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollInstantiateMsg {
    pub generator: Addr,
    pub deposit_amount: Uint128,
    pub poll_name: String,
    pub start_time: u64,
    pub bet_end_time: u64,
}
