use crate::state::{Config, State};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admins: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    RegisterTokenContract {
        token_contract: String,
        creation_deposit: Uint128,
    },
    UpdateConfig {
        creation_deposit: Option<Uint128>,
        reclaimable_threshold: Option<Uint128>,
        new_owner: Option<String>,
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
