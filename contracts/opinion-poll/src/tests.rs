#[cfg(test)]
mod prediction_poll_tests {
    use crate::entrypoints::{execute, instantiate, query};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UserVoteResponse};
    use crate::state::{Config, State};
    use config::config::PollType;
    use cosmwasm_std::{from_binary, Addr, Timestamp, Uint128};

    const DEFAULT_RECLAIMABLE_THRESHOLD: Uint128 = Uint128::new(100);
    const DEPOSIT_AMOUNT: Uint128 = Uint128::new(1_000);

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Opinion,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: None,
            tax_percentage: None,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: Config = from_binary(&res).unwrap();
        let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
        let state: State = from_binary(&res).unwrap();
        assert_eq!(Addr::unchecked("generator"), config.generator);
        assert_eq!("test_poll", config.poll_name);
        assert_eq!(1653673600, config.bet_end_time);
        assert_eq!(DEPOSIT_AMOUNT, state.deposit_amount);
    }

    #[test]
    fn proper_vote() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1649673600);

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Opinion,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: None,
            tax_percentage: None,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("user", &[]);
        let msg = ExecuteMsg::Vote { side: 0 };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserVote {
                address: "user".to_string(),
            },
        )
        .unwrap();
        let value: UserVoteResponse = from_binary(&res).unwrap();
        assert_eq!(0u64, value.side.unwrap());

        // user not voted
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserVote {
                address: "user_not_voted".to_string(),
            },
        )
        .unwrap();
        let value: UserVoteResponse = from_binary(&res).unwrap();
        assert!(value.side.is_none());
    }

    #[test]
    fn proper_finish() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1649673600);

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Opinion,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: None,
            tax_percentage: None,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Vote { side: 0 };
        let info = mock_info("user1", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Vote { side: 1 };
        let info = mock_info("user2", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Vote { side: 1 };
        let info = mock_info("user3", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        env.block.time = Timestamp::from_seconds(2000000000);

        let msg = ExecuteMsg::FinishPoll {};
        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
        let value: State = from_binary(&res).unwrap();
        assert_eq!(Some(&1u64), value.winning_side.unwrap().get(0));
    }

    #[test]
    fn proper_finish_with_multiple_winners() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1649673600);

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Opinion,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: None,
            tax_percentage: None,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Vote { side: 0 };
        let info = mock_info("user1", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Vote { side: 1 };
        let info = mock_info("user2", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Vote { side: 1 };
        let info = mock_info("user3", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Vote { side: 0 };
        let info = mock_info("user4", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        env.block.time = Timestamp::from_seconds(2000000000);

        let msg = ExecuteMsg::FinishPoll {};
        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
        let value: State = from_binary(&res).unwrap();
        assert_eq!(vec![0u64, 1u64], value.winning_side.unwrap());
    }

    #[test]
    fn change_config() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.height = 6340000;

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Opinion,
            bet_end_time: 6400000,
            resolution_time: 6400000,
            minimum_bet_amount: None,
            tax_percentage: None,
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // transfer_owner
        let msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(
            "creator",
            from_binary::<Config>(&res).unwrap().owner.as_str()
        );

        let msg = ExecuteMsg::TransferOwner {
            new_owner: String::from("user1"),
        };
        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), env, msg).unwrap();
        assert_eq!("user1", from_binary::<Config>(&res).unwrap().owner.as_str());
    }
}
