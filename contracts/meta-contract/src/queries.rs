use cosmwasm_std::{Deps, Order, StdResult};
#[cfg(not(feature = "library"))]
use std::str;

use crate::msg::{ConfigResponse, ContractsResponse, StateResponse};
use crate::state::{read_config, read_state, CONTRACTS};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;
    Ok(config)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = read_state(deps.storage)?;
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
