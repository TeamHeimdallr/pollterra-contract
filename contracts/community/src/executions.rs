use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;

use crate::state::{Allowance, ContractConfig, ContractState};

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admins: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let mut config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new().add_attribute("action", "update_config");

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

pub fn increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut state = ContractState::load(deps.storage)?;
    let free_balance = state
        .load_balance(&deps.querier, &env, &config.managing_token)?
        .free_balance;
    if free_balance < amount {
        return Err(ContractError::Std(StdError::generic_err(
            "Insufficient balance",
        )));
    }

    let mut response = Response::new().add_attribute("action", "increase_allowance");

    let address = deps.api.addr_validate(address.as_str())?;
    let mut allowance = Allowance::load_or_default(deps.storage, &address)?;

    allowance.increase(amount);
    allowance.save(deps.storage)?;

    state.remain_allowance_amount += amount;
    state.save(deps.storage)?;

    response = response.add_attribute("address", address.to_string());
    response = response.add_attribute("amount", amount);
    response = response.add_attribute("allowed_amount", allowance.allowed_amount);
    response = response.add_attribute("remain_amount", allowance.remain_amount);

    Ok(response)
}

pub fn decrease_allowance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let config = ContractConfig::load(deps.storage)?;
    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut response = Response::new().add_attribute("action", "decrease_allowance");

    let address = deps.api.addr_validate(address.as_str())?;
    let mut allowance = Allowance::load(deps.storage, &address)?;
    let mut state = ContractState::load(deps.storage)?;

    let amount = if let Some(amount) = amount {
        if allowance.remain_amount < amount {
            return Err(ContractError::Std(StdError::generic_err(
                "Insufficient remain amount",
            )));
        } else {
            amount
        }
    } else {
        allowance.remain_amount
    };

    allowance.decrease(amount)?;
    allowance.save_or_delete(deps.storage)?;

    state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
    state.save(deps.storage)?;

    response = response.add_attribute("address", address.to_string());
    response = response.add_attribute("amount", amount.to_string());
    response = response.add_attribute("allowed_amount", allowance.allowed_amount);
    response = response.add_attribute("remain_amount", allowance.remain_amount);

    Ok(response)
}

pub fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = ContractConfig::load(deps.storage)?;
    let mut state = ContractState::load(deps.storage)?;

    if amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut response = Response::new().add_attribute("action", "transfer");

    let remain_amount = if config.is_admin(&info.sender) {
        let balance = state.load_balance(&deps.querier, &env, &config.managing_token)?;

        if balance.free_balance < amount {
            return Err(ContractError::Std(StdError::generic_err(
                "Insufficient balance",
            )));
        }

        balance.free_balance
    } else {
        let allowance = Allowance::may_load(deps.storage, &info.sender)?;

        if let Some(mut allowance) = allowance {
            if allowance.remain_amount < amount {
                return Err(ContractError::ExceedLimit {});
            }

            allowance.remain_amount = allowance.remain_amount.checked_sub(amount)?;
            allowance.save_or_delete(deps.storage)?;

            state.remain_allowance_amount = state.remain_allowance_amount.checked_sub(amount)?;
            state.save(deps.storage)?;

            allowance.remain_amount
        } else {
            return Err(ContractError::Unauthorized {});
        }
    };

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
