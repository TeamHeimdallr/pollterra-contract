use cosmwasm_std::{Addr, Order, QuerierWrapper, StdResult, Uint128};
use cw20::Cw20QueryMsg;
use cw_storage_plus::Bound;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_LIMIT: u32 = 30;
pub const DEFAULT_LIMIT: u32 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderBy {
    Asc,
    Desc,
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

pub struct RangeOption {
    pub limit: usize,
    pub min: Option<Bound>,
    pub max: Option<Bound>,
    pub order_by: Order,
}

pub fn addr_range_option(
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> RangeOption {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_after = start_after.map(Bound::exclusive);
    let (min, max, order_by) = match order_by {
        Some(OrderBy::Asc) => (start_after, None, Order::Ascending),
        _ => (None, start_after, Order::Descending),
    };

    RangeOption {
        limit,
        min,
        max,
        order_by,
    }
}
