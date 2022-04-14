#[cfg(test)]
mod distributor_tests {
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{
        attr, from_binary, to_binary, Binary, CosmosMsg, DepsMut, SubMsg, Uint128, WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;

    use testutils::mock_querier::mock_dependencies;

    use crate::entrypoints;
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::query_msgs::{ContractConfigResponse, DistributionsResponse, QueryMsg};

    const POLLTERRA_TOKEN: &str = "pollterra_token";
    const CREATOR: &str = "creator";
    const ADMIN_0: &str = "admin0";
    const ADMIN_1: &str = "admin1";
    const NOT_ADMIN: &str = "not_admin";
    const NEW_ADMIN: &str = "new_admin";
    const RECIPIENT: &str = "recipient";
    const RECIPIENT_2: &str = "recipient2";

    fn mock_instantiate(deps: DepsMut) {
        let msg = InstantiateMsg {
            admins: vec![ADMIN_0.to_string(), ADMIN_1.to_string()],
            managing_token: POLLTERRA_TOKEN.to_string(),
        };

        let info = mock_info(CREATOR, &[]);
        let _res = entrypoints::instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    #[test]
    fn test_instantiation() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        let msg = QueryMsg::Config {};
        let res: ContractConfigResponse =
            from_binary(&entrypoints::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

        assert_eq!(POLLTERRA_TOKEN, &res.managing_token);
        assert_eq!(ADMIN_0, res.admins.get(0).unwrap());
        assert_eq!(ADMIN_1, res.admins.get(1).unwrap());
    }

    #[test]
    fn update_admins() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        let info = mock_info(ADMIN_0, &[]);
        let admins = Some(vec![ADMIN_0.to_string(), NEW_ADMIN.to_string()]);
        let msg = ExecuteMsg::UpdateAdmins { admins };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);

        let msg = QueryMsg::Config {};
        let res: ContractConfigResponse =
            from_binary(&entrypoints::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

        assert_eq!(POLLTERRA_TOKEN, &res.managing_token);
        assert_eq!(ADMIN_0, res.admins.get(0).unwrap());
        assert_ne!(ADMIN_1, res.admins.get(1).unwrap());
        assert_eq!(NEW_ADMIN, res.admins.get(1).unwrap());
    }

    #[test]
    fn update_admins_failed_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        let info = mock_info(NOT_ADMIN, &[]);
        let admins = Some(vec![ADMIN_0.to_string(), NEW_ADMIN.to_string()]);
        let msg = ExecuteMsg::UpdateAdmins { admins };

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }

    #[test]
    fn proper_register_distribution() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(20_000u128))],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let start_height = 10000;
        let end_height = 30000;
        let amount = Uint128::new(10_000);

        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: None,
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);

        let msg = QueryMsg::Distributions {};

        let mut env = mock_env();
        env.block.height = 20000;

        let res: DistributionsResponse =
            from_binary(&entrypoints::query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();

        assert_eq!(&start_height, &res.distributions[0].start_height);
        assert_eq!(&end_height, &res.distributions[0].end_height);
        assert_eq!(&RECIPIENT, &res.distributions[0].recipient);
        assert_eq!(&amount, &res.distributions[0].amount);
        assert_eq!(
            amount.multiply_ratio(env.block.height - start_height, end_height - start_height),
            res.distributions[0].distributable_amount
        );
        assert_eq!(&Uint128::new(0), &res.distributions[0].distributed_amount);
        assert_eq!(
            amount.multiply_ratio(env.block.height - start_height, end_height - start_height),
            res.distributions[0].released_amount
        );
    }

    #[test]
    fn register_distribution_failed_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(20_000u128))],
        )]);

        let info = mock_info(NOT_ADMIN, &[]);
        let start_height = 10000;
        let end_height = 30000;
        let amount = Uint128::new(10_000);

        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: None,
        };
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }

    #[test]
    fn proper_update_distribution() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(20_000u128))],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let start_height = 20000;
        let end_height = 40000;
        let amount = Uint128::new(10_000);

        let new_start_height = 30000;
        let new_end_height = 50000;
        let new_amount = Uint128::new(20_000);

        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: None,
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);
        let distribution_id: u64 = res.unwrap().attributes[1].value.parse().unwrap();

        let msg = ExecuteMsg::UpdateDistribution {
            id: distribution_id,
            start_height: Some(new_start_height),
            end_height: Some(new_end_height),
            amount: Some(new_amount),
            message: None,
        };

        let info = mock_info(ADMIN_0, &[]);
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);

        let mut env = mock_env();
        env.block.height = 40000;

        let msg = QueryMsg::Distributions {};
        let res: DistributionsResponse =
            from_binary(&entrypoints::query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();

        assert_eq!(&new_start_height, &res.distributions[0].start_height);
        assert_eq!(&new_end_height, &res.distributions[0].end_height);
        assert_eq!(&RECIPIENT, &res.distributions[0].recipient);
        assert_eq!(&new_amount, &res.distributions[0].amount);
        assert_eq!(
            new_amount.multiply_ratio(
                env.block.height - new_start_height,
                new_end_height - new_start_height
            ),
            res.distributions[0].distributable_amount
        );
        assert_eq!(&Uint128::new(0), &res.distributions[0].distributed_amount);
        assert_eq!(
            new_amount.multiply_ratio(
                env.block.height - new_start_height,
                new_end_height - new_start_height
            ),
            res.distributions[0].released_amount
        );
    }

    #[test]
    fn update_distribution_failed_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(20_000u128))],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let start_height = 20000;
        let end_height = 40000;
        let amount = Uint128::new(10_000);

        let new_start_height = 30000;
        let new_end_height = 50000;
        let new_amount = Uint128::new(20_000);

        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: None,
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);
        let distribution_id: u64 = res.unwrap().attributes[1].value.parse().unwrap();

        let msg = ExecuteMsg::UpdateDistribution {
            id: distribution_id,
            start_height: Some(new_start_height),
            end_height: Some(new_end_height),
            amount: Some(new_amount),
            message: None,
        };

        let info = mock_info(NOT_ADMIN, &[]);
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }

    #[test]
    fn proper_distribute() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(20_000u128))],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let start_height = 10000;
        let end_height = 30000;
        let amount = Uint128::new(10_000);

        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: None,
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);
        let distribution_id: u64 = res.unwrap().attributes[1].value.parse().unwrap();

        let msg = ExecuteMsg::Distribute {
            id: Some(distribution_id),
        };
        let mut env = mock_env();
        env.block.height = 20000;

        let info = mock_info(ADMIN_0, &[]);
        let res = entrypoints::execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(
            &res.attributes,
            &vec![
                attr("action", "distribute"),
                attr(
                    "distribution",
                    format!("{}/{}/{}", distribution_id, RECIPIENT, amount,)
                )
            ]
        );
        let distributed =
            amount.multiply_ratio(env.block.height - start_height, end_height - start_height);

        let send_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: POLLTERRA_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: RECIPIENT.to_string(),
                amount: distributed,
            })
            .unwrap(),
        });
        assert_eq!(&res.messages, &vec![SubMsg::new(send_msg)]);

        let msg = QueryMsg::Distributions {};
        let res: DistributionsResponse =
            from_binary(&entrypoints::query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();

        assert_eq!(&start_height, &res.distributions[0].start_height);
        assert_eq!(&end_height, &res.distributions[0].end_height);
        assert_eq!(&RECIPIENT, &res.distributions[0].recipient);
        assert_eq!(&amount, &res.distributions[0].amount);
        assert_eq!(
            amount.multiply_ratio(env.block.height - start_height, end_height - start_height)
                - distributed,
            res.distributions[0].distributable_amount
        );
        assert_eq!(distributed, res.distributions[0].distributed_amount);
        assert_eq!(
            amount.multiply_ratio(env.block.height - start_height, end_height - start_height),
            res.distributions[0].released_amount
        );
    }

    #[test]
    fn proper_transfer() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        let initial_amount = Uint128::from(20_000u128);
        let send_amount = Uint128::from(10_000u128);

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &initial_amount)],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let start_height = 10000;
        let end_height = 30000;
        let amount = Uint128::new(5_000);

        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: None,
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT_2.to_string(),
            amount: send_amount,
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            &res.attributes,
            &vec![
                attr("action", "transfer"),
                attr("requester", ADMIN_0),
                attr("recipient", RECIPIENT_2),
                attr("amount", send_amount),
                attr("remain_amount", initial_amount - amount),
            ]
        );

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: POLLTERRA_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: RECIPIENT_2.to_string(),
                amount: send_amount,
            })
            .unwrap(),
        });
        assert_eq!(&res.messages, &vec![SubMsg::new(msg)]);
    }

    #[test]
    fn transfer_failed_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        let initial_amount = Uint128::from(20_000u128);
        let send_amount = Uint128::from(10_000u128);

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &initial_amount)],
        )]);

        let msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT_2.to_string(),
            amount: send_amount,
        };

        let info = mock_info(NOT_ADMIN, &[]);
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }

    #[test]
    fn proper_remove_distribution_message() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(20_000u128))],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let start_height = 20000;
        let end_height = 40000;
        let amount = Uint128::new(10_000);

        let attached_msg: Option<Binary> = Some(
            to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(1),
            })
            .unwrap(),
        );

        // register distribution w/ message
        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: attached_msg.clone(),
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);
        let distribution_id: u64 = res.unwrap().attributes[1].value.parse().unwrap();

        // distribute w/ message
        let msg = ExecuteMsg::Distribute {
            id: Some(distribution_id),
        };
        let mut env = mock_env();
        env.block.height = 30000;

        let info = mock_info(ADMIN_0, &[]);
        let res = entrypoints::execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        assert_eq!(
            &res.attributes,
            &vec![
                attr("action", "distribute"),
                attr(
                    "distribution",
                    format!("{}/{}/{}", distribution_id, RECIPIENT, amount,)
                )
            ]
        );
        let distributed_1 =
            amount.multiply_ratio(env.block.height - start_height, end_height - start_height);

        let send_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: POLLTERRA_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: RECIPIENT.to_string(),
                amount: distributed_1,
                msg: attached_msg.unwrap(),
            })
            .unwrap(),
            funds: vec![],
        });

        assert_eq!(res.messages[0], SubMsg::new(send_msg));

        // remove disdtribution message
        let msg = ExecuteMsg::RemoveDistributionMessage {
            id: distribution_id,
        };

        let info = mock_info(ADMIN_0, &[]);
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            &res.attributes,
            &vec![
                attr("action", "remove_distribution_message"),
                attr("is_updated_message", "true")
            ]
        );

        // distribute after removing distribution messages
        let msg = ExecuteMsg::Distribute {
            id: Some(distribution_id),
        };
        let mut env = mock_env();
        env.block.height = 40000;

        let info = mock_info(ADMIN_0, &[]);
        let res = entrypoints::execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(
            &res.attributes,
            &vec![
                attr("action", "distribute"),
                attr(
                    "distribution",
                    format!("{}/{}/{}", distribution_id, RECIPIENT, amount,)
                )
            ]
        );
        let distributed_2 = amount
            .multiply_ratio(env.block.height - start_height, end_height - start_height)
            - distributed_1;

        let send_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: POLLTERRA_TOKEN.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: RECIPIENT.to_string(),
                amount: distributed_2,
            })
            .unwrap(),
        });

        assert_eq!(res.messages[0], SubMsg::new(send_msg));
    }

    #[test]
    fn remove_distribution_message_failed_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(20_000u128))],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let start_height = 20000;
        let end_height = 40000;
        let amount = Uint128::new(10_000);

        let attached_msg: Option<Binary> = Some(
            to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(1),
            })
            .unwrap(),
        );

        // register distribution w/ message
        let msg = ExecuteMsg::RegisterDistribution {
            start_height,
            end_height,
            recipient: RECIPIENT.to_string(),
            amount,
            message: attached_msg,
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg);
        let distribution_id: u64 = res.unwrap().attributes[1].value.parse().unwrap();

        // remove disdtribution message
        let msg = ExecuteMsg::RemoveDistributionMessage {
            id: distribution_id,
        };

        let info = mock_info(NOT_ADMIN, &[]);
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }
}
