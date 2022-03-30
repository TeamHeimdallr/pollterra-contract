use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Order,
    Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use protobuf::Message;
#[cfg(not(feature = "library"))]
use std::str;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ContractsResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{read_state, store_state, Cw20HookMsg, State, CONTRACTS, STATE};

use messages::msg::PollInstantiateMsg;

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
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        token_contract: String::new(),
        creation_deposit: Uint128::zero(),
        reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
        num_contract: 0,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

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
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, _env, info, msg),
        ExecuteMsg::RegisterTokenContract {
            token_contract,
            creation_deposit,
        } => register_token_contract(deps, info, token_contract, creation_deposit),
        ExecuteMsg::UpdateCreationDeposit { creation_deposit } => {
            update_creation_deposit(deps, info, creation_deposit)
        }
        ExecuteMsg::UpdateReclaimableThreshold {
            reclaimable_threshold,
        } => update_reclaimable_threshold(deps, info, reclaimable_threshold),
        ExecuteMsg::TransferOwner { new_owner } => try_transfer_owner(deps, info, new_owner),
    }
}

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
            start_time,
            bet_end_time,
        }) => init_poll(
            deps,
            info,
            code_id,
            cw20_msg.sender,
            cw20_msg.amount,
            poll_name,
            start_time,
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
    start_time: u64,
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
            start_time,
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        INSTANTIATE_REPLY_ID => after_poll_init(deps, msg),
        _ => Err(StdError::generic_err("invalid reply id")),
    }
}

fn after_poll_init(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    let reply_result = msg.result.unwrap();
    let data = reply_result.data.unwrap();
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(data.as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let contract_address = res.get_contract_address();

    let addr = &deps.api.addr_validate(contract_address)?;
    let _ = CONTRACTS.save(deps.storage, addr, &());

    let event_vec: Vec<Event> = reply_result.events;

    let mut deposit_amount: Option<String> = None;
    for event in event_vec.iter() {
        if "wasm".eq_ignore_ascii_case(&event.ty) {
            for attribute in event.attributes.iter() {
                if "deposit_amount".eq_ignore_ascii_case(&attribute.key) {
                    deposit_amount = Some(attribute.value.clone());
                }
            }
        }
    }

    if deposit_amount.is_none() {
        panic!(""); // TODO error message
    }
    let deposit_amount = Uint128::from(deposit_amount.unwrap().parse::<u128>().unwrap());

    let state: State = read_state(deps.storage).unwrap();

    Ok(Response::new()
        .add_attribute("method", "reply")
        .add_attribute("contract_address", contract_address)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.token_contract,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: contract_address.to_string(),
                amount: deposit_amount,
            })?,
            funds: vec![],
        })))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::GetContracts {} => to_binary(&query_contracts(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_state(deps.storage)?;
    Ok(state)
}

fn query_contracts(deps: Deps) -> StdResult<ContractsResponse> {
    let contracts: StdResult<Vec<_>> = CONTRACTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (k, _) = item?;
            let addr = deps.api.addr_validate(str::from_utf8(&k)?)?;
            Ok(addr)
        })
        .collect();
    Ok(ContractsResponse {
        contracts: contracts.unwrap(),
    })
}
