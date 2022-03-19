use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response,
    StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use protobuf::Message;
#[cfg(not(feature = "library"))]
use std::str;
// use terra_cosmwasm::TerraQuerier;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ContractsResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::response::MsgInstantiateContractResponse;
use crate::state::{read_state, store_state, State, CONTRACTS, STATE};

use messages::msg::PollInstantiateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:meta-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// reply_id is only one for now
const INSTANTIATE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
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
        ExecuteMsg::InitPoll {
            code_id,
            poll_name,
            start_time,
            bet_end_time,
        } => try_init_poll(deps, info, code_id, poll_name, start_time, bet_end_time),
        ExecuteMsg::TransferOwner { new_owner } => try_transfer_owner(deps, info, new_owner),
    }
}

pub fn try_init_poll(
    deps: DepsMut,
    info: MessageInfo,
    code_id: u64,
    poll_name: String,
    start_time: u64,
    bet_end_time: u64,
) -> StdResult<Response> {
    let owner: Addr = read_state(deps.storage).unwrap().owner;

    if info.sender != owner {
        return Err(StdError::generic_err("only the owner can init a poll"));
    }

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(owner.to_string()),
        code_id,
        msg: to_binary(&PollInstantiateMsg {
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
    // We have only one type of reply so don't need to use `match msg.id` for now
    let data = msg.result.unwrap().data.unwrap();
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(data.as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let contract_address = res.get_contract_address();

    let addr = &deps.api.addr_validate(contract_address)?;

    let _ = CONTRACTS.save(deps.storage, addr, &());

    Ok(Response::new()
        .add_attribute("method", "reply")
        .add_attribute("contract_address", contract_address))
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::ReplyOn;

    #[test]
    fn proper_init_poll() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::InitPoll {
            code_id: 0u64,
            poll_name: "test_poll_name".to_string(),
            start_time: 6300000,
            bet_end_time: 6400000,
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            res.messages,
            vec![SubMsg {
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: 0u64,
                    msg: to_binary(&PollInstantiateMsg {
                        poll_name: "test_poll_name".to_string(),
                        start_time: 6300000,
                        bet_end_time: 6400000,
                    })
                    .unwrap(),
                    funds: vec![],
                    label: "test_poll_name".to_string(),
                    admin: Some("creator".to_string()),
                }),
                gas_limit: None,
                id: INSTANTIATE_REPLY_ID,
                reply_on: ReplyOn::Success,
            }]
        );

        let sub_msg_vec = res.messages;
        for sub_msg in sub_msg_vec.iter() {
            println!("{:#?}", sub_msg);
        }
    }
}
