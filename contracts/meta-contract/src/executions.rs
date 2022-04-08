use crate::error::ContractError;
use crate::msg::{Cw20HookMsg, OpinionPollExecuteMsg, PredictionPollExecuteMsg};
use crate::state::Config;
use config::config::PollType;
use cosmwasm_std::{
    from_binary, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg, Uint128, WasmMsg,
};

use cw20::Cw20ReceiveMsg;

use messages::msg::PollInstantiateMsg;

// reply_id is only one for now
pub const INSTANTIATE_REPLY_ID: u64 = 1;

pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = Config::load(deps.storage)?;

    if config.token_contract != deps.api.addr_validate(info.sender.as_str())? {
        return Err(ContractError::IncorrectTokenContract {});
    }

    let creation_deposit: Uint128 = config.creation_deposit;
    if creation_deposit > cw20_msg.amount {
        return Err(ContractError::InsufficientTokenDeposit(creation_deposit));
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::InitPoll {
            code_id,
            poll_name,
            poll_type,
            bet_end_time,
            resolution_time,
            poll_admin,
        }) => init_poll(
            deps,
            info,
            code_id,
            cw20_msg.sender,
            cw20_msg.amount,
            poll_name,
            poll_type,
            bet_end_time,
            resolution_time,
            poll_admin,
        ),
        _ => Err(ContractError::InvalidCw20Msg {}),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn init_poll(
    deps: DepsMut,
    _info: MessageInfo,
    code_id: u64,
    generator: String,
    deposit_amount: Uint128,
    poll_name: String,
    poll_type: String,
    bet_end_time: u64,
    resolution_time: u64,
    poll_admin: Option<String>,
) -> Result<Response, ContractError> {
    let config = Config::load(deps.storage)?;

    if config.creation_deposit != deposit_amount {
        return Err(ContractError::InvalidTokenDeposit(config.creation_deposit));
    }

    let poll_type = match poll_type.as_str() {
        "prediction" => Ok(PollType::Prediction),
        "opinion" => Ok(PollType::Opinion),
        _ => Err(ContractError::InvalidPollType {}),
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: poll_admin,
        code_id,
        msg: to_binary(&PollInstantiateMsg {
            generator: deps.api.addr_validate(&generator)?,
            token_contract: config.token_contract,
            deposit_amount,
            reclaimable_threshold: config.reclaimable_threshold,
            poll_name: poll_name.clone(),
            poll_type: poll_type?,
            bet_end_time,
            resolution_time,
            minimum_bet_amount: Some(config.minimum_bet_amount),
            tax_percentage: Some(config.tax_percentage),
        })?,
        funds: vec![],
        label: poll_name,
    });

    let submsg = SubMsg::reply_on_success(msg, INSTANTIATE_REPLY_ID);

    Ok(Response::new()
        .add_attribute("method", "try_init_poll")
        .add_submessage(submsg))
}

pub fn register_token_contract(
    deps: DepsMut,
    info: MessageInfo,
    token_contract: String,
    creation_deposit: Uint128,
) -> Result<Response, ContractError> {
    let mut config = Config::load(deps.storage)?;

    if !String::new().eq(&config.token_contract) {
        return Err(ContractError::TokenAlreadyRegistered {});
    }

    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    config.token_contract = deps.api.addr_validate(&token_contract)?.to_string();
    config.creation_deposit = creation_deposit;
    config.save(deps.storage)?;

    Ok(Response::new().add_attribute("method", "register_token_contract"))
}

pub fn finish_poll(
    deps: DepsMut,
    info: MessageInfo,
    poll_contract: String,
    poll_type: String,
    winner: Option<u64>,
) -> Result<Response, ContractError> {
    let config = Config::load(deps.storage)?;

    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let poll_type = match poll_type.as_str() {
        "prediction" => Ok(PollType::Prediction),
        "opinion" => Ok(PollType::Opinion),
        _ => Err(ContractError::InvalidPollType {}),
    }?;

    if poll_type == PollType::Prediction && winner.is_none() {
        return Err(ContractError::EmptyWinner {});
    }

    let message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: poll_contract,
        msg: match poll_type {
            PollType::Prediction => to_binary(&PredictionPollExecuteMsg::FinishPoll {
                winner: winner.unwrap(),
            })?,
            PollType::Opinion => to_binary(&OpinionPollExecuteMsg::FinishPoll {})?,
        },
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(message)
        .add_attribute("method", "finish_poll"))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    creation_deposit: Option<Uint128>,
    reclaimable_threshold: Option<Uint128>,
    new_admins: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let mut config = Config::load(deps.storage)?;

    if !config.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(creation_deposit) = creation_deposit {
        if String::new().eq(&config.token_contract) {
            return Err(ContractError::TokenNotRegistered {});
        }
        config.creation_deposit = creation_deposit;
    }

    if let Some(reclaimable_threshold) = reclaimable_threshold {
        config.reclaimable_threshold = reclaimable_threshold;
    }

    if let Some(new_admins) = new_admins.as_ref() {
        config.admins = new_admins
            .iter()
            .map(|v| deps.api.addr_validate(v))
            .collect::<StdResult<Vec<Addr>>>()?;
    }

    config.save(deps.storage)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}
