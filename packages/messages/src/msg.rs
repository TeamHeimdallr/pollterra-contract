use config::config::PollType;
use cosmwasm_std::{Addr, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollInstantiateMsg {
    pub generator: Addr,
    pub token_contract: String,
    pub deposit_amount: Uint128,
    pub reclaimable_threshold: Uint128,
    pub poll_name: String,
    pub poll_type: PollType,
    pub end_time: u64,
    pub num_side: u64,
    // only for prediction poll
    pub resolution_time: Option<u64>,
    pub minimum_bet_amount: Option<Uint128>,
    pub tax_percentage: Option<Decimal>,
    // TODO : participation requirements for opinion poll
}
