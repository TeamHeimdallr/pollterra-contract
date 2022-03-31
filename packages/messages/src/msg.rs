use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollInstantiateMsg {
    pub generator: Addr,
    pub token_contract: String,
    pub deposit_amount: Uint128,
    pub reclaimable_threshold: Uint128,
    pub poll_name: String,
    pub bet_end_time: u64,
}
