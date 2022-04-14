#[cfg(test)]
mod community_tests {
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{from_binary, to_binary, CosmosMsg, DepsMut, Uint128, WasmMsg};
    use cw20::Cw20ExecuteMsg;

    use testutils::mock_querier::mock_dependencies;

    use crate::entrypoints;
    use crate::error::ContractError;
    use messages::community::execute_msgs::{ExecuteMsg, InstantiateMsg};
    use messages::community::query_msgs::{AllowanceResponse, ContractConfigResponse, QueryMsg};

    const POLLTERRA_TOKEN: &str = "pollterra_token";
    const CREATOR: &str = "creator";
    const ADMIN_0: &str = "admin0";
    const ADMIN_1: &str = "admin1";
    const NEW_ADMIN: &str = "new_admin";
    const NON_ADMIN: &str = "non_admin";
    const RECEIVER: &str = "receiver";
    const RECIPIENT: &str = "recipient";

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

        let info = mock_info(NON_ADMIN, &[]);
        let admins = Some(vec![ADMIN_0.to_string(), NEW_ADMIN.to_string()]);
        let msg = ExecuteMsg::UpdateAdmins { admins };
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }

    #[test]
    fn increase_allowance() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::from(1_000u128),
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = QueryMsg::Allowance {
            address: RECEIVER.to_string(),
        };
        let res: AllowanceResponse =
            from_binary(&entrypoints::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

        assert_eq!(RECEIVER, &res.address);
        assert_eq!(&Uint128::from(1_000u128), &res.allowed_amount);
        assert_eq!(&Uint128::from(1_000u128), &res.remain_amount);
    }

    #[test]
    fn increase_allowance_failed_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let info = mock_info(NON_ADMIN, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::from(1_000u128),
        };

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }

    #[test]
    fn increase_allowance_failed_zero_amount() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::zero(),
        };

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InvalidZeroAmount {})
        ));
    }

    #[test]
    fn increase_allowance_failed_insufficient_balance() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        let _balance = Uint128::from(1_000_000u128);

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &_balance)],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::from(2_000_000u128),
        };

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InsufficientFreeBalance(_balance))
        ));
    }

    #[test]
    fn decrease_allowance() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::from(1_000u128),
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::DecreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Some(Uint128::from(300u128)),
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = QueryMsg::Allowance {
            address: RECEIVER.to_string(),
        };
        let res: AllowanceResponse =
            from_binary(&entrypoints::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

        assert_eq!(RECEIVER, &res.address);
        assert_eq!(&Uint128::from(700u128), &res.allowed_amount);
        assert_eq!(&Uint128::from(700u128), &res.remain_amount);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::DecreaseAllowance {
            address: RECEIVER.to_string(),
            amount: None,
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = QueryMsg::Allowance {
            address: RECEIVER.to_string(),
        };
        let res: AllowanceResponse =
            from_binary(&entrypoints::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

        assert_eq!(RECEIVER, &res.address);
        assert_eq!(&Uint128::zero(), &res.allowed_amount);
        assert_eq!(&Uint128::zero(), &res.remain_amount);
    }

    #[test]
    fn decrease_allowance_failed_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::from(1_000u128),
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(NON_ADMIN, &[]);
        let msg = ExecuteMsg::DecreaseAllowance {
            address: RECEIVER.to_string(),
            amount: None,
        };
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::Unauthorized {})
        ));
    }

    #[test]
    fn decrease_allowance_failed_insufficient() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let _amount = Uint128::from(1_000u128);
        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: _amount,
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::DecreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Some(Uint128::from(3_000u128)),
        };
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InsufficientRemainAmount(_amount))
        ));
    }

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::from(500_000u128),
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(RECEIVER, &[]);
        let msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::from(300_000u128),
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: POLLTERRA_TOKEN.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: RECIPIENT.to_string(),
                    amount: Uint128::from(300_000u128),
                })
                .unwrap(),
                funds: vec![],
            })
        );

        let msg = QueryMsg::Allowance {
            address: RECEIVER.to_string(),
        };
        let res: AllowanceResponse =
            from_binary(&entrypoints::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

        assert_eq!(RECEIVER, &res.address);
        assert_eq!(&Uint128::from(500_000u128), &res.allowed_amount);
        assert_eq!(&Uint128::from(200_000u128), &res.remain_amount);

        // ADMIN can transfer without allowance
        let info = mock_info(ADMIN_1, &[]);
        let msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::from(500_000u128),
        };
        let res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: POLLTERRA_TOKEN.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: RECIPIENT.to_string(),
                    amount: Uint128::from(500_000u128),
                })
                .unwrap(),
                funds: vec![],
            })
        );
    }

    #[test]
    fn transfer_failed_amount_zero() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: Uint128::from(500_000u128),
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(RECEIVER, &[]);
        let msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::zero(),
        };

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InvalidZeroAmount {})
        ));
    }

    #[test]
    fn transfer_failed_admin_insufficient() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        let _init_balance = Uint128::from(1_000_000u128);
        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &_init_balance)],
        )]);

        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::from(2_000_000u128),
        };
        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InsufficientFreeBalance(_init_balance))
        ));
    }

    #[test]
    fn transfer_failed_non_admin_insufficient() {
        let mut deps = mock_dependencies(&[]);
        mock_instantiate(deps.as_mut());

        deps.querier.with_token_balances(&[(
            &POLLTERRA_TOKEN.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(1_000_000u128),
            )],
        )]);

        let _allowance_amount = Uint128::from(500_000u128);
        let info = mock_info(ADMIN_0, &[]);
        let msg = ExecuteMsg::IncreaseAllowance {
            address: RECEIVER.to_string(),
            amount: _allowance_amount,
        };
        let _res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(RECEIVER, &[]);
        let msg = ExecuteMsg::Transfer {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::from(700_000u128),
        };

        assert!(matches!(
            entrypoints::execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InsufficientRemainAmount(_allowance_amount))
        ));
    }
}
