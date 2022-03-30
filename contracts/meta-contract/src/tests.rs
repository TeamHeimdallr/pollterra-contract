#[cfg(test)]
mod meta_contract_tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{
        to_binary, Binary, ContractResult, CosmosMsg, Event, Reply, ReplyOn, SubMsg,
        SubMsgExecutionResponse, Uint128, WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;
    use protobuf::Message;
    use base64::encode;

    use crate::contract::{execute, instantiate, query, reply, INSTANTIATE_REPLY_ID};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::response::MsgInstantiateContractResponse;
    use crate::state::Cw20HookMsg;

    const TOKEN_CONTRACT: &str = "pollterra";
    const DEPOSIT_AMOUNT: Uint128 = Uint128::new(1_000);

    #[test]
    fn after_poll_init() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::RegisterTokenContract {
            token_contract: TOKEN_CONTRACT.to_string(),
            creation_deposit: DEPOSIT_AMOUNT,
        };
        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let mut reply_message = MsgInstantiateContractResponse::default();
        reply_message.set_contract_address("contract_address".to_string());

        let aa = Message::write_to_bytes(&reply_message).unwrap();
        let bb = Binary::from(aa);

        let _reply: Reply = Reply {
            id: INSTANTIATE_REPLY_ID,
            result: ContractResult::Ok(SubMsgExecutionResponse {
                events: vec![Event::new("").add_attribute("deposit_amount", DEPOSIT_AMOUNT)],
                data: Some(bb),
            }),
        };

        let aa = to_binary(&Cw20HookMsg::InitPoll {
            code_id: 58425,
            poll_name: "testpoll0".to_string(),
            start_time: 100000000000,
            bet_end_time: 200000000000,
        }).unwrap().to_base64();

        assert_eq!(to_binary(&Uint128::zero()).unwrap().to_base64(), aa);
        

        let res = reply(deps.as_mut(), mock_env(), _reply).unwrap();
        assert_eq!(res.messages.len(), 1);
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TOKEN_CONTRACT.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "contract_address".to_string(),
                    amount: DEPOSIT_AMOUNT,
                })
                .unwrap(),
                funds: vec![],
            })
        );
    }
}
