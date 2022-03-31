use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;
#[cfg(not(feature = "library"))]
use std::str;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use crate::{executions, queries, replies};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:meta-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// reply_id is only one for now
pub const INSTANTIATE_REPLY_ID: u64 = 1;

const DEFAULT_RECLAIMABLE_THRESHOLD: Uint128 = Uint128::new(1_000);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        token_contract: String::new(),
        creation_deposit: Uint128::zero(),
        reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
        num_contract: 0,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => executions::receive_cw20(deps, _env, info, msg),
        ExecuteMsg::RegisterTokenContract {
            token_contract,
            creation_deposit,
        } => executions::register_token_contract(deps, info, token_contract, creation_deposit),
        ExecuteMsg::UpdateCreationDeposit { creation_deposit } => {
            executions::update_creation_deposit(deps, info, creation_deposit)
        }
        ExecuteMsg::UpdateReclaimableThreshold {
            reclaimable_threshold,
        } => executions::update_reclaimable_threshold(deps, info, reclaimable_threshold),
        ExecuteMsg::TransferOwner { new_owner } => {
            executions::try_transfer_owner(deps, info, new_owner)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_REPLY_ID => replies::after_poll_init(deps, msg),
        _ => Err(StdError::generic_err("invalid reply id")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::GetContracts {} => to_binary(&queries::query_contracts(deps)?),
    }
}
