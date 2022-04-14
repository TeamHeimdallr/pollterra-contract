use cosmwasm_std::{Deps, Order, StdResult};
#[cfg(not(feature = "library"))]
use std::str;

use messages::meta_contract::query_msgs::{ConfigResponse, ContractsResponse, StateResponse};
use messages::meta_contract::state::{Config, State, CONTRACTS};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = Config::load(deps.storage)?;
    Ok(config)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = State::load(deps.storage)?;
    Ok(state)
}

pub fn query_contracts(deps: Deps) -> StdResult<ContractsResponse> {
    let contracts: StdResult<Vec<_>> = CONTRACTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (k, _) = item?;
            let addr = deps.api.addr_validate(str::from_utf8(&k)?)?;
            Ok(addr)
        })
        .collect();
    Ok(ContractsResponse {
        contracts: contracts.unwrap(),
    })
}
