#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::query_msgs::QueryMsg;
use crate::state::{ContractConfig, ContractState};
use crate::{contract, queries};

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

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { admins } => contract::update_config(deps, env, info, admins),
        ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient,
            amount,
            message,
        } => contract::register_distribution(
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
        } => contract::update_distribution(
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
            contract::remove_distribution_message(deps, env, info, id)
        }
        ExecuteMsg::Distribute { id } => contract::distribute(deps, env, info, id),
        ExecuteMsg::Transfer { recipient, amount } => {
            contract::transfer(deps, env, info, recipient, amount)
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
