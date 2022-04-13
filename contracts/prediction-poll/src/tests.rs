#[cfg(test)]
mod prediction_poll_tests {
    use crate::entrypoints::{execute, instantiate, query};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};

    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UserBetResponse, UserRewardsResponse};
    use crate::state::{Config, State};
    use config::config::PollType;
    use cosmwasm_std::{
        coins, from_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Timestamp, Uint128,
    };

    const DENOM: &str = "uusd";
    const DEFAULT_MINIMUM_BET: Uint128 = Uint128::new(1_000);
    const DEFAULT_RECLAIMABLE_THRESHOLD: Uint128 = Uint128::new(1_000);
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
            poll_type: PollType::Prediction,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: Some(DEFAULT_MINIMUM_BET),
            tax_percentage: Some(Decimal::zero()),
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
    fn proper_bet_once() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1649673600);

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Prediction,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: Some(DEFAULT_MINIMUM_BET),
            tax_percentage: Some(Decimal::zero()),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("user", &coins(1_000_000, DENOM));
        let msg = ExecuteMsg::Bet { side: 0 };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserBet {
                address: "user".to_string(),
                side: 0,
            },
        )
        .unwrap();
        let value: UserBetResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(1_000_000), value.amount);

        let info = mock_info("user0", &coins(DEFAULT_MINIMUM_BET.u128() - 1, DENOM));
        let msg = ExecuteMsg::Bet { side: 1 };
        let res = execute(deps.as_mut(), env, info, msg);
        assert!(res.is_err());
    }

    #[test]
    fn proper_revert() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1649673600);

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Prediction,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: Some(DEFAULT_MINIMUM_BET),
            tax_percentage: Some(Decimal::zero()),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user1", &coins(1_000_000, DENOM));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user2", &coins(2_000_000, DENOM));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 1 };
        let info = mock_info("user2", &coins(8_000_000, DENOM));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::RevertPoll {};
        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserBet {
                address: "user1".to_string(),
                side: 0,
            },
        )
        .unwrap();
        let value: UserBetResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(1_000_000), value.amount);
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
            poll_type: PollType::Prediction,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: Some(DEFAULT_MINIMUM_BET),
            tax_percentage: Some(Decimal::percent(1_u64)),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user1", &coins(1_000_000, DENOM));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 1 };
        let info = mock_info("user2", &coins(2_000_000, DENOM));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        env.block.time = Timestamp::from_seconds(2000000000);
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(3_000_000, DENOM));

        let msg = ExecuteMsg::FinishPoll { winner: 0 };
        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserRewards {
                address: "user1".to_string(),
            },
        )
        .unwrap();
        let value: UserRewardsResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(2980000), value.reward);
    }

    #[test]
    fn proper_claim() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1649673600);

        let msg = InstantiateMsg {
            generator: Addr::unchecked("generator"),
            token_contract: "terra1pollterratoken".to_string(),
            deposit_amount: DEPOSIT_AMOUNT,
            reclaimable_threshold: DEFAULT_RECLAIMABLE_THRESHOLD,
            poll_name: "test_poll".to_string(),
            poll_type: PollType::Prediction,
            bet_end_time: 1653673600,
            resolution_time: 1653673600,
            minimum_bet_amount: Some(DEFAULT_MINIMUM_BET),
            tax_percentage: Some(Decimal::percent(1_u64)),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user1", &coins(1_000_000, DENOM));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 1 };
        let info = mock_info("user2", &coins(2_000_000, DENOM));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        env.block.time = Timestamp::from_seconds(2000000000);
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(3_000_000, DENOM));

        let msg = ExecuteMsg::FinishPoll { winner: 0 };
        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserRewards {
                address: "user1".to_string(),
            },
        )
        .unwrap();
        let value: UserRewardsResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(2980000), value.reward);

        let msg = ExecuteMsg::Claim {};
        let info = mock_info("user1", &[]);
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".to_string(),
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128::new(2980000)
                }]
            }),
            res.messages[0].msg
        );

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserRewards {
                address: "user1".to_string(),
            },
        )
        .unwrap();
        let value: UserRewardsResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(0), value.reward);
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
            poll_type: PollType::Prediction,
            bet_end_time: 6400000,
            resolution_time: 6400000,
            minimum_bet_amount: Some(DEFAULT_MINIMUM_BET),
            tax_percentage: Some(Decimal::zero()),
        };

        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // set_minimum_bet
        let msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(
            DEFAULT_MINIMUM_BET,
            from_binary::<Config>(&res).unwrap().minimum_bet_amount
        );

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::SetMinimumBet { amount: 2_000u128 };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = QueryMsg::Config {};
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(
            Uint128::from(2_000u128),
            from_binary::<Config>(&res).unwrap().minimum_bet_amount
        );

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
