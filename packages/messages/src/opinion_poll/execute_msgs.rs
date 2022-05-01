use crate::msg::PollInstantiateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

pub type InstantiateMsg = PollInstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Vote { side: u64 },
    FinishPoll {},
    // TODO : only for internal QA
    ForceFinishPoll {},
    ReclaimDeposit {},
    TransferOwner { new_owner: String },
}
