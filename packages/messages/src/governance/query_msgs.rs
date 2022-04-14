use super::state::{OrderBy, PollStatus};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Config returns the configuration values of the governance contract
    Config {},
    // State returns the governance state values such as the poll_count and the amount deposited
    State {},
    // Staker returns Staked governance token information for the provided address
    Staker {
        address: String,
    },
    // Poll returns the information related to a Poll if that poll exists
    Poll {
        poll_id: u64,
    },
    Polls {
        filter: Option<PollStatus>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    // Voters returns a defined range of voters for a given poll denoted by its poll_id and its range defined or limited using start_after
    Voters {
        poll_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}
