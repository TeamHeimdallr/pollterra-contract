#[cfg(test)]
mod community_tests {
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{from_binary, DepsMut};

    use testutils::mock_querier::mock_dependencies;

    use crate::entrypoints;
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::query_msgs::{ContractConfigResponse, QueryMsg};

    const POLLTERRA_TOKEN: &str = "pollterra_token";
    const CREATOR: &str = "creator";
    const ADMIN_0: &str = "admin0";
    const ADMIN_1: &str = "admin1";
    const NEW_ADMIN: &str = "new_admin";

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
}
