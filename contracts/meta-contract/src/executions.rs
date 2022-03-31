use crate::state::{read_state, store_state, Cw20HookMsg, State};
use cosmwasm_std::{
    from_binary, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg,
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
) -> Result<Response, StdError> {
    let state: State = read_state(deps.storage).unwrap();
    if state.token_contract != deps.api.addr_validate(info.sender.as_str())? {
        return Err(StdError::generic_err("Incorrect token contract"));
    }

    let creation_deposit: Uint128 = state.creation_deposit;
    if creation_deposit > cw20_msg.amount {
        return Err(StdError::generic_err("Insufficient token amount"));
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::InitPoll {
            code_id,
            poll_name,
            bet_end_time,
        }) => init_poll(
            deps,
            info,
            code_id,
            cw20_msg.sender,
            cw20_msg.amount,
            poll_name,
            bet_end_time,
        ),
        _ => Err(StdError::generic_err("Cw20Msg doesn't match")),
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
    bet_end_time: u64,
) -> StdResult<Response> {
    let state: State = read_state(deps.storage).unwrap();
    let contract_owner: Addr = state.owner;

    if state.creation_deposit != deposit_amount {
        return Err(StdError::generic_err(format!(
            "deposit amount should be {}",
            state.creation_deposit
        )));
    }

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(contract_owner.to_string()),
        code_id,
        msg: to_binary(&PollInstantiateMsg {
            generator: deps.api.addr_validate(&generator)?,
            token_contract: state.token_contract,
            deposit_amount,
            reclaimable_threshold: state.reclaimable_threshold,
            poll_name: poll_name.clone(),
            bet_end_time,
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
) -> StdResult<Response> {
    let mut state: State = read_state(deps.storage).unwrap();
    if !String::new().eq(&state.token_contract) {
        return Err(StdError::generic_err("already registered"));
    }

    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can register a token contract",
        ));
    }

    state.token_contract = deps.api.addr_validate(&token_contract)?.to_string();
    state.creation_deposit = creation_deposit;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "register_token_contract"))
}

// TODO : update config at once
pub fn update_creation_deposit(
    deps: DepsMut,
    info: MessageInfo,
    creation_deposit: Uint128,
) -> StdResult<Response> {
    let mut state: State = read_state(deps.storage).unwrap();
    if String::new().eq(&state.token_contract) {
        return Err(StdError::generic_err("token not registered"));
    }

    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can update creation deposit amount",
        ));
    }

    state.creation_deposit = creation_deposit;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "update_creatoin_deposit"))
}

pub fn update_reclaimable_threshold(
    deps: DepsMut,
    info: MessageInfo,
    reclaimable_threshold: Uint128,
) -> StdResult<Response> {
    let mut state: State = read_state(deps.storage).unwrap();

    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can update reclaimable threshold amount",
        ));
    }

    state.reclaimable_threshold = reclaimable_threshold;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "update_reclaimable_threshold"))
}

pub fn try_transfer_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;
    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can transfer the ownership",
        ));
    }
    state.owner = deps.api.addr_validate(&new_owner)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "try_transfer_owner"))
}
