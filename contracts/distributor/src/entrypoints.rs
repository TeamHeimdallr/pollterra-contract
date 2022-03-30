#[cfg(not(feature = "library"))]
use cw2::set_contract_version;

use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::query_msgs::QueryMsg;
use crate::state::{ContractConfig, ContractState};
use crate::{executions, queries};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:distributor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
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
        distribution_count: 0,
        locked_amount: Uint128::zero(),
        distributed_amount: Uint128::zero(),
    }
    .save(deps.storage)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
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
        ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient,
            amount,
            message,
        } => executions::register_distribution(
            deps,
            env,
            info,
            start_height,
            end_height,
            recipient,
            amount,
            message,
        ),
        ExecuteMsg::UpdateDistribution {
            id,
            start_height,
            end_height,
            amount,
            message,
        } => executions::update_distribution(
            deps,
            env,
            info,
            id,
            start_height,
            end_height,
            amount,
            message,
        ),
        ExecuteMsg::RemoveDistributionMessage { id } => {
            executions::remove_distribution_message(deps, env, info, id)
        }
        ExecuteMsg::Distribute { id } => executions::distribute(deps, env, info, id),
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
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&queries::get_config(deps, env)?)?),
        QueryMsg::State {} => Ok(to_binary(&queries::get_state(deps, env)?)?),
        QueryMsg::Distributions {} => Ok(to_binary(&queries::get_distributions(deps, env)?)?),
    }
}
