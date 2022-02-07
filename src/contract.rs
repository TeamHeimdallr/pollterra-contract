use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Timestamp, Uint128,
};
use cw2::set_contract_version;
#[cfg(not(feature = "library"))]
use std::str;
// use terra_cosmwasm::TerraQuerier;

use crate::error::ContractError;
use crate::msg::{
    BetLiveResponse, BetStatusResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    RewardLiveResponse, UserBetResponse, UserRewardsResponse,
};
use crate::state::{
    read_state, store_state, BetStatus, State, BETS, REWARDS, SIDE_TOTAL_AMOUNT, STATE,
    USER_TOTAL_AMOUNT,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:pollterra-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // TODO: change owner
    let state = State {
        owner: info.sender.clone(),
        status: BetStatus::Created,
        bet_live: false,
        reward_live: false,
        poll_name: msg.poll_name,
        start_time: msg.start_time,
        bet_end_time: msg.bet_end_time,
        cancel_hold: msg.cancel_hold,
        total_amount: Uint128::new(0),
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
        ExecuteMsg::Bet { side } => try_bet(deps, _env, info, side),
        ExecuteMsg::CancelBet { side } => try_cancel_bet(deps, _env, info, side),
        ExecuteMsg::FinishPoll { winner } => try_finish_poll(deps, _env, info, winner),
        ExecuteMsg::RevertPoll {} => try_revert_poll(deps, info),
        ExecuteMsg::Claim {} => try_claim(deps, info),
    }
}

pub fn try_bet(deps: DepsMut, env: Env, info: MessageInfo, side: u8) -> StdResult<Response> {
    let addr = info.sender.clone();

    let mut state = read_state(deps.storage)?;

    // current block height is less than start height or larger than bet end height
    if env.block.height < state.start_time || env.block.height >= state.bet_end_time {
        // update bet live state
        state.bet_live = false;
        return Err(StdError::generic_err(format!("Bet is not live. current block height: {}, start_block_height: {}, bet_end_block_height: {}", env.block.height, state.start_time, state.bet_end_time)));
    }

    // update bet live state
    state.bet_live = true;

    // Check if some funds are sent
    let sent = match info.funds.len() {
        0 => Err(StdError::generic_err(
            "you need to send some ust in order to bet",
        )),
        1 => {
            if info.funds[0].denom == "uusd" {
                Ok(info.funds[0].amount)
            } else {
                Err(StdError::generic_err(
                    "you need to send ust in order to bet",
                ))
            }
        }
        _ => Err(StdError::generic_err("Only send ust to bet")),
    }?;

    // sent 0 ust case
    if sent.is_zero() {
        return Err(StdError::generic_err(
            "you need to send some ust in order to bet",
        ));
    }

    // add bet amount to BETS state (accumulate single side)
    BETS.update(
        deps.storage,
        (&side.to_be_bytes(), &addr),
        |exists| -> StdResult<Uint128> {
            match exists {
                Some(bet) => {
                    let mut modified = bet;
                    modified += sent;
                    Ok(modified)
                }
                None => Ok(sent),
            }
        },
    )?;

    // add bet amount to USER_TOTAL_AMOUNT state (accumulate both side)
    USER_TOTAL_AMOUNT.update(deps.storage, &addr, |exists| -> StdResult<Uint128> {
        match exists {
            Some(bet) => {
                let mut modified = bet;
                modified += sent;
                Ok(modified)
            }
            None => Ok(sent),
        }
    })?;

    // add bet amount to SIDE_TOTAL_AMOUNT state (accumulate single side, every user)
    SIDE_TOTAL_AMOUNT.update(
        deps.storage,
        &side.to_be_bytes(),
        |exists| -> StdResult<Uint128> {
            match exists {
                Some(bet) => {
                    let mut modified = bet;
                    modified += sent;
                    Ok(modified)
                }
                None => Ok(sent),
            }
        },
    )?;

    // Save the new state
    state.total_amount += sent;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "try_bet"),
        ("address", info.sender.as_str()),
        ("side", &side.to_string()),
        ("amount", &sent.to_string()),
    ]))
}

pub fn try_cancel_bet(deps: DepsMut, env: Env, info: MessageInfo, side: u8) -> StdResult<Response> {
    let addr = info.sender;

    let mut state = read_state(deps.storage)?;

    // exceeds cancel threshold block height
    if env.block.height > state.cancel_hold {
        return Err(StdError::generic_err(format!(
            "cannot cancel after {} block height",
            state.cancel_hold
        )));
    }

    // BETS State load
    let value = match BETS.may_load(deps.storage, (&side.to_be_bytes(), &addr))? {
        None => Uint128::zero(),
        Some(amount) => amount,
    };

    if value == Uint128::zero() {
        return Err(StdError::generic_err("there's no bet to cancel"));
    }

    BETS.update(
        deps.storage,
        (&side.to_be_bytes(), &addr),
        |_exists| -> StdResult<Uint128> { Ok(Uint128::zero()) },
    )?;

    USER_TOTAL_AMOUNT.update(deps.storage, &addr, |exists| -> StdResult<Uint128> {
        match exists {
            Some(bet) => {
                let mut modified = bet;
                modified -= value;
                Ok(modified)
            }
            None => Ok(Uint128::zero()),
        }
    })?;

    SIDE_TOTAL_AMOUNT.update(
        deps.storage,
        &side.to_be_bytes(),
        |exists| -> StdResult<Uint128> {
            match exists {
                Some(bet) => {
                    let mut modified = bet;
                    modified -= value;
                    Ok(modified)
                }
                None => Ok(Uint128::zero()),
            }
        },
    )?;

    state.total_amount -= value;
    store_state(deps.storage, &state)?;

    // TODO: deduct tax?
    Ok(Response::new()
        .add_attribute("method", "try_cancel_bet")
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: addr.to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: value,
            }],
        })))
}

pub fn try_finish_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    winner: u8,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    // only contract's owner can finish poll
    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can finish poll",
        ));
    }

    // already finished
    if state.status == BetStatus::Closed || state.status == BetStatus::Reward {
        return Err(StdError::generic_err("already finished poll"));
    }

    // cannot finish before poll ends
    if env.block.time < Timestamp::from_seconds(state.bet_end_time) {
        return Err(StdError::generic_err(
            "bet is live now, The poll cannot be finished before the bet ends",
        ));
    }

    let winner_amount = match SIDE_TOTAL_AMOUNT.may_load(deps.storage, &winner.to_be_bytes())? {
        Some(value) => value,
        None => Uint128::new(0),
    };

    let odds = Decimal::from_ratio(state.total_amount, winner_amount);
    // println!("{}", winner_amount);
    // println!("{}", odds);

    // iterate over them all
    let all: StdResult<Vec<_>> = BETS
        .prefix(&winner.to_be_bytes())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, reward) = item?;
            // println!("{}", str::from_utf8(&addr)?);
            Ok((addr, reward))
        })
        .collect();

    for (addr, reward) in all?.iter() {
        REWARDS.update(
            deps.storage,
            &deps.api.addr_validate(str::from_utf8(addr)?)?,
            |_exists| -> StdResult<Uint128> {
                Ok(((*reward) * odds) * (Decimal::percent(99_u64))) // 1% fee
            },
        )?;
        // println!("{}", ((*reward) * odds) * (Decimal::percent(99 as u64)) );
    }

    // Save the new state
    state.status = BetStatus::Reward;
    state.bet_live = false;
    state.reward_live = true;
    state.bet_end_time = 0;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "try_finish_poll"))
}

pub fn try_revert_poll(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    // TODO: make sure all of users didn't claim rewards

    // only contract's owner can revert
    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can revert poll",
        ));
    }

    // already reverted
    if state.status == BetStatus::Closed {
        return Err(StdError::generic_err("already reverted poll"));
    }

    let mut msgs: Vec<CosmosMsg> = vec![];

    // iterate over them all
    let _all: StdResult<Vec<_>> = USER_TOTAL_AMOUNT
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, spent) = item?;
            // println!("{}", str::from_utf8(&addr)?);
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: str::from_utf8(&addr)?.to_string(),
                amount: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: spent,
                }],
            }));
            Ok((addr, spent))
        })
        .collect();

    if msgs.is_empty() {
        return Ok(Response::new().add_attribute("method", "try_revert_poll"));
    }

    // update bet status
    state.status = BetStatus::Closed;
    state.bet_live = false;
    state.reward_live = false;
    state.bet_end_time = 0;

    // TODO? USER_TOTAL_AMOUNT reset? not necessary

    // Save the new state
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "try_revert_poll")
        .add_messages(msgs))
}

pub fn try_claim(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let addr = info.sender;
    let state = read_state(deps.storage)?;

    if state.status != BetStatus::Reward {
        return Err(StdError::generic_err(format!(
            "cannot cliam rewards. current status: {}",
            state.status
        )));
    }

    // REWARDS State load
    let value = match REWARDS.may_load(deps.storage, &addr)? {
        None => Uint128::zero(),
        Some(amount) => amount,
    };

    if value == Uint128::zero() {
        return Err(StdError::generic_err("there's no rewards to claim"));
    }

    REWARDS.update(deps.storage, &addr, |_exists| -> StdResult<Uint128> {
        Ok(Uint128::zero())
    })?;

    Ok(Response::new()
        .add_attribute("method", "try_claim")
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: addr.to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: value,
            }],
        })))
}

// static DECIMAL_FRACTION: u128 = 1_000_000_000_000_000_000u128;
// fn deduct_tax(deps: DepsMut, coins: Vec<Coin>) -> StdResult<Vec<Coin>> {
//     let terra_querier = TerraQuerier::new(&deps.querier);
//     let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;

//     coins
//         .into_iter()
//         .map(|v| {
//             let tax_cap: Uint128 = (terra_querier.query_tax_cap(v.denom.to_string())?).cap;

//             Ok(Coin {
//                 amount: Uint128::from(
//                     v.amount.u128()
//                         - std::cmp::min(
//                             v.amount.multiply_ratio(
//                                 DECIMAL_FRACTION,
//                                 (tax_rate * DECIMAL_FRACTION.into()).u128() + DECIMAL_FRACTION,
//                             ),
//                             tax_cap,
//                         )
//                         .u128(),
//                 ),
//                 denom: v.denom,
//             })
//         })
//         .collect()
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::BetStatus {} => to_binary(&query_bet_status(deps, _env)?),
        QueryMsg::BetLive {} => to_binary(&query_bet_live(deps, _env)?),
        QueryMsg::RewardLive {} => to_binary(&query_reward_live(deps)?),
        QueryMsg::UserBet { address, side } => to_binary(&query_user_bet(deps, address, side)?),
        QueryMsg::UserRewards { address } => to_binary(&query_user_rewards(deps, address)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_state(deps.storage)?;
    Ok(state)
}

fn query_bet_status(deps: Deps, env: Env) -> StdResult<BetStatusResponse> {
    let state = read_state(deps.storage)?;

    // bet_live can be incorrect if "bet" execution is not called. threfore use block height
    let bet_status = if state.status == BetStatus::Closed {
        BetStatus::Closed
    } else if env.block.height < state.start_time {
        BetStatus::Created
    } else if env.block.height < state.bet_end_time {
        BetStatus::Betting
    } else if state.reward_live {
        BetStatus::Reward
    } else {
        BetStatus::BetHold
    };

    Ok(BetStatusResponse { status: bet_status })
}

fn query_bet_live(deps: Deps, env: Env) -> StdResult<BetLiveResponse> {
    let is_bet_live = query_bet_status(deps, env)?.status == BetStatus::Betting;

    Ok(BetLiveResponse {
        bet_live: is_bet_live,
    })
}

fn query_reward_live(deps: Deps) -> StdResult<RewardLiveResponse> {
    let state = read_state(deps.storage)?;
    Ok(RewardLiveResponse {
        reward_live: state.reward_live,
    })
}

fn query_user_bet(deps: Deps, address: String, side: u8) -> StdResult<UserBetResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let value = match BETS.may_load(deps.storage, (&side.to_be_bytes(), &addr))? {
        None => Uint128::new(0),
        Some(amount) => amount,
    };

    Ok(UserBetResponse { amount: value })
}

fn query_user_rewards(deps: Deps, address: String) -> StdResult<UserRewardsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let value = match REWARDS.may_load(deps.storage, &addr)? {
        None => Uint128::new(0),
        Some(amount) => amount,
    };

    Ok(UserRewardsResponse { reward: value })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            poll_name: "test_poll".to_string(),
            start_time: 6300000,
            bet_end_time: 6400000,
            cancel_hold: 6390000,
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let value: State = from_binary(&res).unwrap();
        assert_eq!("test_poll", value.poll_name);
        assert_eq!(6300000, value.start_time);
        assert_eq!(6400000, value.bet_end_time);
        assert_eq!(6390000, value.cancel_hold);
    }

    #[test]
    fn proper_bet_once() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.height = 6340000;

        let msg = InstantiateMsg {
            poll_name: "test_poll".to_string(),
            start_time: 6300000,
            bet_end_time: 6400000,
            cancel_hold: 6390000,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let msg = ExecuteMsg::Bet { side: 0 };

        let info = mock_info("user", &coins(1_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

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
    }

    #[test]
    fn proper_revert() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.height = 6340000;

        let msg = InstantiateMsg {
            poll_name: "test_poll".to_string(),
            start_time: 6300000,
            bet_end_time: 6400000,
            cancel_hold: 6390000,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user1", &coins(1_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user2", &coins(2_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 1 };
        let info = mock_info("user2", &coins(8_000_000, "uusd"));
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
    fn proper_cancel() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.height = 6340000;

        let msg = InstantiateMsg {
            poll_name: "test_poll".to_string(),
            start_time: 6300000,
            bet_end_time: 6400000,
            cancel_hold: 6390000,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user1", &coins(1_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::CancelBet { side: 0 };
        let info = mock_info("user1", &[]);
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
        assert_eq!(Uint128::new(0), value.amount);
    }

    #[test]
    fn proper_finish() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.height = 6340000;

        let msg = InstantiateMsg {
            poll_name: "test_poll".to_string(),
            start_time: 6300000,
            bet_end_time: 6400000,
            cancel_hold: 6390000,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user1", &coins(1_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 1 };
        let info = mock_info("user2", &coins(2_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

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
        assert_eq!(Uint128::new(2970000), value.reward);
    }

    #[test]
    fn proper_claim() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.height = 6340000;

        let msg = InstantiateMsg {
            poll_name: "test_poll".to_string(),
            start_time: 6300000,
            bet_end_time: 6400000,
            cancel_hold: 6390000,
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 0 };
        let info = mock_info("user1", &coins(1_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::Bet { side: 1 };
        let info = mock_info("user2", &coins(2_000_000, "uusd"));
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

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
        assert_eq!(Uint128::new(2970000), value.reward);

        let msg = ExecuteMsg::Claim {};
        let info = mock_info("user1", &[]);
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".to_string(),
                amount: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(2970000)
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
}
