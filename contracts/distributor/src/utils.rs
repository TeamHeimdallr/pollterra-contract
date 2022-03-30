use cosmwasm_std::{Addr, QuerierWrapper, StdResult, Uint128};
use cw20::Cw20QueryMsg;

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
