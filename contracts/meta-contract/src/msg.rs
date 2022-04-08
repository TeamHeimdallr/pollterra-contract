use crate::state::{Config, State};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    RegisterTokenContract {
        token_contract: String,
        creation_deposit: Uint128,
    },
    FinishPoll {
        poll_contract: String,
        poll_type: String,
        winner: Option<u64>,
    },
    UpdateConfig {
        creation_deposit: Option<Uint128>,
        reclaimable_threshold: Option<Uint128>,
        new_admins: Option<Vec<String>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    GetContracts {},
}

pub type ConfigResponse = Config;
pub type StateResponse = State;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractsResponse {
    pub contracts: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    InitPoll {
        code_id: u64,
        poll_name: String,
        poll_type: String,
        bet_end_time: u64,
        resolution_time: u64,
        poll_admin: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PredictionPollExecuteMsg {
    FinishPoll { winner: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpinionPollExecuteMsg {
    FinishPoll {},
}
