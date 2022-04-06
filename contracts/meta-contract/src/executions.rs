use crate::error::ContractError;
use crate::state::{read_config, store_config, Config, Cw20HookMsg};
use config::config::PollType;
use cosmwasm_std::{
    from_binary, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128,
    WasmMsg,
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
    let config: Config = read_config(deps.storage).unwrap();
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
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage).unwrap();
    let contract_owner: Addr = config.owner;

    if config.creation_deposit != deposit_amount {
        return Err(ContractError::InsufficientTokenDeposit(
            config.creation_deposit,
        ));
    }

    let poll_type = match poll_type.as_str() {
        "prediction" => Ok(PollType::Prediction),
        "opinion" => Ok(PollType::Opinion),
        _ => Err(ContractError::InvalidPollType {}),
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(contract_owner.to_string()),
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
    let mut config: Config = read_config(deps.storage).unwrap();
    if !String::new().eq(&config.token_contract) {
        return Err(ContractError::TokenAlreadyRegistered {});
    }

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.token_contract = deps.api.addr_validate(&token_contract)?.to_string();
    config.creation_deposit = creation_deposit;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "register_token_contract"))
}

// TODO : update config at once
pub fn update_creation_deposit(
    deps: DepsMut,
    info: MessageInfo,
    creation_deposit: Uint128,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage).unwrap();
    if String::new().eq(&config.token_contract) {
        return Err(ContractError::TokenNotRegistered {});
    }

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.creation_deposit = creation_deposit;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "update_creatoin_deposit"))
}

pub fn update_reclaimable_threshold(
    deps: DepsMut,
    info: MessageInfo,
    reclaimable_threshold: Uint128,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage).unwrap();

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.reclaimable_threshold = reclaimable_threshold;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "update_reclaimable_threshold"))
}

pub fn try_transfer_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    config.owner = deps.api.addr_validate(&new_owner)?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "try_transfer_owner"))
}
