use crate::error::ContractError;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Event, Reply, Response, StdError, Uint128, WasmMsg,
};

use cw20::Cw20ExecuteMsg;
use protobuf::Message;

use crate::response::MsgInstantiateContractResponse;
use crate::state::{Config, State, CONTRACTS};

pub fn after_poll_init(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let reply_result = msg.result.unwrap();
    let data = reply_result.data.unwrap();
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(data.as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let contract_address = res.get_contract_address();

    let addr = &deps.api.addr_validate(contract_address)?;
    CONTRACTS.save(deps.storage, addr, &())?;
    let mut state: State = State::load(deps.storage)?;
    state.num_contract += 1;
    state.save(deps.storage)?;

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
        return Err(ContractError::InsufficientBalance {});
    }
    let deposit_amount = Uint128::from(deposit_amount.unwrap().parse::<u128>().unwrap());

    let config: Config = Config::load(deps.storage).unwrap();

    Ok(Response::new()
        .add_attribute("method", "reply")
        .add_attribute("contract_address", contract_address)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.token_contract,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: contract_address.to_string(),
                amount: deposit_amount,
            })?,
            funds: vec![],
        })))
}
