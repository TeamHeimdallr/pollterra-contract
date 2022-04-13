#[cfg(test)]
mod community_tests {
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{from_binary, DepsMut, Uint128};

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
}
