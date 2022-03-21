use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Uint128};
use cw20::{Cw20QueryMsg, Denom};

pub fn query_balance(querier: &QuerierWrapper, denom: Denom, address: Addr) -> StdResult<Uint128> {
    match denom {
        Denom::Native(denom) => querier.query_balance(address, denom).map(|v| v.amount),
        Denom::Cw20(contract_addr) => query_cw20_balance(querier, &contract_addr, &address),
    }
}

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    let response: cw20::BalanceResponse = querier.query_wasm_smart(
        contract_addr,
        &Cw20QueryMsg::Balance {
            address: account_addr.to_string(),
        },
    )?;

    Ok(response.balance)
}
