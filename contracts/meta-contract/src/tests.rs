#[cfg(test)]
mod meta_contract_tests {
    use crate::error::ContractError;
    use messages::meta_contract::state::Config;

    use config::config::PollType;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{
        attr, to_binary, Binary, ContractResult, CosmosMsg, Decimal, Event, Reply, SubMsg,
        SubMsgExecutionResponse, Uint128, WasmMsg,
    };
    use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
    use protobuf::Message;

    use crate::entrypoints;
    use crate::response::MsgInstantiateContractResponse;
    use messages::meta_contract::execute_msgs::{Cw20HookMsg, ExecuteMsg, InstantiateMsg};
    use messages::msg::PollInstantiateMsg;

    const TOKEN_CONTRACT: &str = "pollterra";
    const DEPOSIT_AMOUNT: Uint128 = Uint128::new(1_000);
    const TEST_CODE_ID: u64 = 1234;
    const INSTANTIATE_REPLY_ID: u64 = 1;
    const DEFAULT_RECLAIMABLE_THRESHOLD: Uint128 = Uint128::new(1_000);

    #[test]
    fn after_poll_init() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            admins: vec!["creator".to_string()],
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::RegisterTokenContract {
            token_contract: TOKEN_CONTRACT.to_string(),
            creation_deposit: DEPOSIT_AMOUNT,
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let mut reply_message = MsgInstantiateContractResponse::default();
        reply_message.set_contract_address("contract_address".to_string());

        let aa = Message::write_to_bytes(&reply_message).unwrap();
        let bb = Binary::from(aa);

        let _reply: Reply = Reply {
            id: entrypoints::INSTANTIATE_REPLY_ID,
            result: ContractResult::Ok(SubMsgExecutionResponse {
                // The event type of InstantiateMsg is 'wasm'
                events: vec![Event::new("wasm").add_attribute("deposit_amount", DEPOSIT_AMOUNT)],
                data: Some(bb),
            }),
        };

        let res = entrypoints::reply(deps.as_mut(), mock_env(), _reply).unwrap();
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

    #[test]
    fn proper_poll_init_with_poll_type() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            admins: vec!["creator".to_string()],
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::RegisterTokenContract {
            token_contract: TOKEN_CONTRACT.to_string(),
            creation_deposit: DEPOSIT_AMOUNT,
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // prediction poll type
        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: TOKEN_CONTRACT.to_string(),
            amount: Uint128::from(1_000u128),
            msg: to_binary(&Cw20HookMsg::InitPoll {
                code_id: TEST_CODE_ID,
                poll_name: "test_poll".to_string(),
                poll_type: "prediction".to_string(),
                end_time: 1653673599,
                resolution_time: Some(1653673600),
                poll_admin: None,
                num_side: None,
            })
            .unwrap(),
        });
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes, vec![attr("method", "try_init_poll"),]);

        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: TEST_CODE_ID,
            msg: to_binary(&PollInstantiateMsg {
                generator: info.sender,
                token_contract: TOKEN_CONTRACT.to_string(),
                deposit_amount: Uint128::from(1_000u128),
                reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
                minimum_bet_amount: Some(Uint128::from(1_000u128)),
                tax_percentage: Some(Decimal::percent(5)),
                poll_name: "test_poll".to_string(),
                poll_type: PollType::Prediction,
                end_time: 1653673599,
                num_side: 2,
                resolution_time: Some(1653673600),
            })
            .unwrap(),
            funds: vec![],
            label: "test_poll".to_string(),
        });
        let submsg = SubMsg::reply_on_success(msg, INSTANTIATE_REPLY_ID);

        assert_eq!(res.messages, vec![submsg]);

        // opinion poll type
        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: TOKEN_CONTRACT.to_string(),
            amount: Uint128::from(1_000u128),
            msg: to_binary(&Cw20HookMsg::InitPoll {
                code_id: TEST_CODE_ID,
                poll_name: "test_poll".to_string(),
                poll_type: "opinion".to_string(),
                end_time: 1653673599,
                num_side: None,
                resolution_time: None,
                poll_admin: None,
            })
            .unwrap(),
        });
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes, vec![attr("method", "try_init_poll"),]);

        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: TEST_CODE_ID,
            msg: to_binary(&PollInstantiateMsg {
                generator: info.sender,
                token_contract: TOKEN_CONTRACT.to_string(),
                deposit_amount: Uint128::from(1_000u128),
                reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
                minimum_bet_amount: Some(Uint128::from(1_000u128)),
                tax_percentage: Some(Decimal::percent(5)),
                poll_name: "test_poll".to_string(),
                poll_type: PollType::Opinion,
                end_time: 1653673599,
                num_side: 2,
                resolution_time: None,
            })
            .unwrap(),
            funds: vec![],
            label: "test_poll".to_string(),
        });
        let submsg = SubMsg::reply_on_success(msg, INSTANTIATE_REPLY_ID);

        assert_eq!(res.messages, vec![submsg]);
    }

    #[test]
    fn fail_poll_init_with_wrong_poll_type() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            admins: vec!["creator".to_string()],
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::RegisterTokenContract {
            token_contract: TOKEN_CONTRACT.to_string(),
            creation_deposit: DEPOSIT_AMOUNT,
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: TOKEN_CONTRACT.to_string(),
            amount: Uint128::from(1_000u128),
            msg: to_binary(&Cw20HookMsg::InitPoll {
                code_id: TEST_CODE_ID,
                poll_name: "test_poll".to_string(),
                poll_type: "Wrong Poll Type".to_string(),
                end_time: 1653673599,
                resolution_time: Some(1653673600),
                poll_admin: None,
                num_side: None,
            })
            .unwrap(),
        });

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InvalidPollType {})
        ));
    }

    #[test]
    fn fail_poll_init_with_resolution_time() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            admins: vec!["creator".to_string()],
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::RegisterTokenContract {
            token_contract: TOKEN_CONTRACT.to_string(),
            creation_deposit: DEPOSIT_AMOUNT,
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: TOKEN_CONTRACT.to_string(),
            amount: Uint128::from(1_000u128),
            msg: to_binary(&Cw20HookMsg::InitPoll {
                code_id: TEST_CODE_ID,
                poll_name: "test_poll".to_string(),
                poll_type: "prediction".to_string(),
                end_time: 1653673599,
                resolution_time: None,
                poll_admin: None,
                num_side: None,
            })
            .unwrap(),
        });

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::ShouldHaveResolutionTime {})
        ));

        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: TOKEN_CONTRACT.to_string(),
            amount: Uint128::from(1_000u128),
            msg: to_binary(&Cw20HookMsg::InitPoll {
                code_id: TEST_CODE_ID,
                poll_name: "test_poll".to_string(),
                poll_type: "prediction".to_string(),
                end_time: 1653673601,
                resolution_time: Some(1653673600),
                poll_admin: None,
                num_side: None,
            })
            .unwrap(),
        });

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::ShouldEndBeforeResolution {})
        ));

        let info = mock_info(TOKEN_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: TOKEN_CONTRACT.to_string(),
            amount: Uint128::from(1_000u128),
            msg: to_binary(&Cw20HookMsg::InitPoll {
                code_id: TEST_CODE_ID,
                poll_name: "test_poll".to_string(),
                poll_type: "opinion".to_string(),
                end_time: 1653673599,
                resolution_time: Some(1653673600),
                poll_admin: None,
                num_side: None,
            })
            .unwrap(),
        });

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::ShouldNotHaveResolutionTime {})
        ));
    }

    #[test]
    fn is_admin_test() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            admins: vec!["admin1".to_string(), "admin2".to_string()],
        };
        let info = mock_info("creator", &[]);
        let _res = entrypoints::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let config = Config::load(&deps.storage).unwrap();

        let info = mock_info("creator", &[]);
        assert!(!config.is_admin(&info.sender));

        let info = mock_info("not-admin", &[]);
        assert!(!config.is_admin(&info.sender));

        let info = mock_info("admin1", &[]);
        assert!(config.is_admin(&info.sender));

        let info = mock_info("admin2", &[]);
        assert!(config.is_admin(&info.sender));
    }
}
