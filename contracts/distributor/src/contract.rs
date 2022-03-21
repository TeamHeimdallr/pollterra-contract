#[cfg(not(feature = "library"))]
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::error::ContractError;

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admins: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    // TODO
    // Not Implemented Yet
    Ok(Response::default())
}

pub fn register_distribution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_height: u64,
    end_height: u64,
    recipient: String,
    amount: Uint128,
    message: Option<Binary>,
) -> Result<Response, ContractError> {
    // TODO
    // Not Implemented Yet
    Ok(Response::default())
}

pub fn update_distribution(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    start_height: Option<u64>,
    end_height: Option<u64>,
    amount: Option<Uint128>,
    message: Option<Binary>,
) -> Result<Response, ContractError> {
    // TODO
    // Not Implemented Yet
    Ok(Response::default())
}

pub fn remove_distribution_message(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    // TODO
    // Not Implemented Yet
    Ok(Response::default())
}

pub fn distribute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    id: Option<u64>,
) -> Result<Response, ContractError> {
    // TODO
    // Not Implemented Yet
    Ok(Response::default())
}

pub fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO
    // Not Implemented Yet
    Ok(Response::default())
}
