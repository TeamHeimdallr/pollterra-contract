#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::query_msgs::QueryMsg;
use crate::state::{ContractConfig, ContractState};

use crate::executions;
use crate::queries;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:community";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let response = Response::new().add_attribute("action", "instantiate");

    ContractConfig {
        admins: msg
            .admins
            .iter()
            .map(|v| deps.api.addr_validate(v))
            .collect::<StdResult<Vec<Addr>>>()?,
        managing_token: deps.api.addr_validate(msg.managing_token.as_str())?,
    }
    .save(deps.storage)?;

    ContractState {
        remain_allowance_amount: Uint128::zero(),
    }
    .save(deps.storage)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmins { admins } => executions::update_admins(deps, env, info, admins),
        ExecuteMsg::IncreaseAllowance { address, amount } => {
            executions::increase_allowance(deps, env, info, address, amount)
        }
        ExecuteMsg::DecreaseAllowance { address, amount } => {
            executions::decrease_allowance(deps, env, info, address, amount)
        }
        ExecuteMsg::Transfer { recipient, amount } => {
            executions::transfer(deps, env, info, recipient, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let result = match msg {
        QueryMsg::Config {} => to_binary(&queries::get_config(deps, env)?),
        QueryMsg::Balance {} => to_binary(&queries::get_balance(deps, env)?),
        QueryMsg::Allowance { address } => to_binary(&queries::get_allowance(deps, env, address)?),
        QueryMsg::Allowances {
            start_after,
            limit,
            order_by,
        } => to_binary(&queries::query_allowances(
            deps,
            env,
            start_after,
            limit,
            order_by,
        )?),
    }?;

    Ok(result)
}
