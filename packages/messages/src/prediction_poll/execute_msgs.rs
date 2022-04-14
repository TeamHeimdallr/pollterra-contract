use crate::msg::PollInstantiateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

pub type InstantiateMsg = PollInstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Bet {
        side: u64,
    },
    FinishPoll {
        winner: u64,
    },
    RevertPoll {},
    Claim {},
    ResetPoll {
        poll_name: String,
        bet_end_time: u64,
    },
    ReclaimDeposit {},
    TransferOwner {
        new_owner: String,
    },
    SetMinimumBet {
        amount: u128,
    },
}
