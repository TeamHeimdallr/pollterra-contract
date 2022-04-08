use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;
#[cfg(not(feature = "library"))]
use std::str;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, State};
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
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Config {
        owner: info.sender.clone(),
        admins: match msg.admins {
            Some(admin_list) => Some(
                admin_list
                    .iter()
                    .map(|v| deps.api.addr_validate(v))
                    .collect::<StdResult<Vec<Addr>>>()?,
            ),
            None => None,
        },
        token_contract: String::new(),
        creation_deposit: Uint128::zero(),
        reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
        minimum_bet_amount: Uint128::from(1_000u128),
        tax_percentage: Decimal::percent(5),
    }
    .save(deps.storage)?;

    State { num_contract: 0 }.save(deps.storage)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => executions::receive_cw20(deps, _env, info, msg),
        ExecuteMsg::RegisterTokenContract {
            token_contract,
            creation_deposit,
        } => executions::register_token_contract(deps, info, token_contract, creation_deposit),
        ExecuteMsg::UpdateConfig {
            creation_deposit,
            reclaimable_threshold,
            new_owner,
            new_admins,
        } => executions::update_config(
            deps,
            info,
            creation_deposit,
            reclaimable_threshold,
            new_owner,
            new_admins,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_REPLY_ID => replies::after_poll_init(deps, msg),
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::GetContracts {} => to_binary(&queries::query_contracts(deps)?),
    }
}
