use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PollInstantiateMsg {
    pub poll_name: String,
    pub start_time: u64,
    pub bet_end_time: u64,
}
