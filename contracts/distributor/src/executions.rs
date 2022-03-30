use cosmwasm_std::{
    attr, to_binary, Addr, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;
use crate::state::{ContractConfig, ContractState, Distribution};
use crate::utils;

pub fn update_admins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admins: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let mut config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new().add_attribute("action", "update_admins");

    if let Some(admins) = admins.as_ref() {
        config.admins = admins
            .iter()
            .map(|v| deps.api.addr_validate(v))
            .collect::<StdResult<Vec<Addr>>>()?;
        response = response.add_attribute("is_updated_admins", "true");
    }

    config.save(deps.storage)?;

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
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
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new().add_attribute("action", "register_distribution");
    let mut state = ContractState::load(deps.storage)?;

    state.distribution_count += 1;

    let distribution = Distribution {
        id: state.distribution_count,
        start_height,
        end_height,
        recipient: deps.api.addr_validate(recipient.as_str())?,
        amount,
        distributed_amount: Uint128::zero(),
        message,
    };
    response
        .attributes
        .push(attr("distribution_id", distribution.id.to_string()));

    distribution.save(deps.storage)?;

    let balance =
        utils::query_cw20_balance(&deps.querier, &config.managing_token, &env.contract.address)?;

    state.lock(balance, amount)?;
    state.save(deps.storage)?;

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
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
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new().add_attribute("action", "update_distribution");

    let mut distribution = Distribution::may_load(deps.storage, id)?
        .ok_or_else(|| StdError::not_found("Distribution"))?;

    let prev_released_amount = distribution.released_amount(env.block.height);

    if let Some(start_height) = start_height {
        distribution.start_height = start_height;
        response = response.add_attribute("is_updated_start_height", "true");
    }

    if let Some(end_height) = end_height {
        distribution.end_height = end_height;
        response = response.add_attribute("is_updated_end_height", "true");
    }

    if let Some(amount) = amount {
        if distribution.released_amount(env.block.height) > amount {
            return Err(ContractError::Std(StdError::generic_err(
                "amount must be greater than released_amount",
            )));
        }

        let mut state = ContractState::load(deps.storage)?;

        if distribution.amount > amount {
            state.unlock(distribution.amount.checked_sub(amount)?)?;
        } else {
            let balance = utils::query_cw20_balance(
                &deps.querier,
                &config.managing_token,
                &env.contract.address,
            )?;

            state.lock(balance, amount.checked_sub(distribution.amount)?)?;
        }
        state.save(deps.storage)?;

        distribution.amount = amount;
        response = response.add_attribute("is_updated_amount", "true");
    }

    if let Some(message) = message {
        distribution.message = Some(message);
        response = response.add_attribute("is_updated_message", "true");
    }

    if prev_released_amount > distribution.released_amount(env.block.height) {
        return Err(ContractError::Std(StdError::generic_err(
            "Can not decrease released_amount",
        )));
    }

    distribution.save(deps.storage)?;

    Ok(response)
}

pub fn remove_distribution_message(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new().add_attribute("action", "remove_distribution_message");

    let mut distribution = Distribution::may_load(deps.storage, id)?
        .ok_or_else(|| StdError::not_found("Distribution"))?;

    distribution.message = None;
    response = response.add_attribute("is_updated_message", "true");

    distribution.save(deps.storage)?;

    Ok(response)
}

pub fn distribute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    id: Option<u64>,
) -> Result<Response, ContractError> {
    let mut distributions = if let Some(id) = id {
        vec![Distribution::may_load(deps.storage, id)?.ok_or_else(|| {
            StdError::generic_err("This id is expired distribution or invalid id")
        })?]
    } else {
        Distribution::load_all(deps.storage)?
    };

    let mut response = Response::new().add_attribute("action", "distribute");

    if !distributions.is_empty() {
        let config = ContractConfig::load(deps.storage)?;
        let mut state = ContractState::load(deps.storage)?;

        for distribution in distributions.iter_mut() {
            let amount = distribution
                .released_amount(env.block.height)
                .checked_sub(distribution.distributed_amount)
                .unwrap_or_else(|_| Uint128::zero());

            if amount.is_zero() {
                continue;
            }

            let send_msg = if let Some(message) = distribution.message.as_ref() {
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.managing_token.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: distribution.recipient.to_string(),
                        amount,
                        msg: message.clone(),
                    })?,
                    funds: vec![],
                })
            } else {
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.managing_token.to_string(),
                    funds: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: distribution.recipient.to_string(),
                        amount,
                    })
                    .unwrap(),
                })
            };

            response.messages.push(SubMsg::new(send_msg));

            state.unlock(amount)?;
            state.distributed_amount += amount;
            distribution.distributed_amount += amount;

            if distribution.amount == distribution.distributed_amount {
                distribution.delete(deps.storage);
            } else {
                distribution.save(deps.storage)?;
            }

            response.attributes.push(attr(
                "distribution",
                format!(
                    "{}/{}/{}",
                    distribution.id, distribution.recipient, distribution.amount,
                ),
            ));
        }

        state.save(deps.storage)?;
    }

    Ok(response)
}

pub fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let state = ContractState::load(deps.storage)?;
    let balance =
        utils::query_cw20_balance(&deps.querier, &config.managing_token, &env.contract.address)?;
    let remain_amount = balance.checked_sub(state.locked_amount)?;

    if remain_amount < amount {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    let mut response = Response::new().add_attribute("action", "transfer");

    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.managing_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: deps.api.addr_validate(&recipient)?.to_string(),
            amount,
        })?,
        funds: vec![],
    }));

    response = response.add_attribute("requester", info.sender.as_str());
    response = response.add_attribute("recipient", recipient);
    response = response.add_attribute("amount", amount);
    response = response.add_attribute("remain_amount", remain_amount);

    Ok(response)
}
