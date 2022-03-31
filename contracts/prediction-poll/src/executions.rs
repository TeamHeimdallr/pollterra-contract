use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use std::str;

use crate::state::{
    read_state, store_state, BetStatus, BETS, REWARDS, SIDE_TOTAL_AMOUNT, STATE, USER_TOTAL_AMOUNT,
};
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:pollterra-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DENOM: &str = "uusd";

pub fn try_bet(deps: DepsMut, env: Env, info: MessageInfo, side: u8) -> StdResult<Response> {
    let addr = info.sender.clone();

    let mut state = read_state(deps.storage)?;

    // current block time is less than start time or larger than bet end time
    if env.block.time >= Timestamp::from_seconds(state.bet_end_time) {
        return Err(StdError::generic_err(format!(
            "Bet is not live. current block time: {}, bet end time: {}",
            env.block.time, state.bet_end_time
        )));
    }

    // Check if some funds are sent
    let sent = match info.funds.len() {
        0 => Err(StdError::generic_err(
            "you need to send some ust in order to bet",
        )),
        1 => {
            if info.funds[0].denom == DENOM {
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

    if sent < state.minimum_bet {
        return Err(StdError::generic_err(format!(
            "The bet amount should be over {}",
            state.minimum_bet
        )));
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
    state.bet_end_time = 0;

    let mut cw20_msg = Cw20ExecuteMsg::Transfer {
        recipient: state.generator.to_string(),
        amount: state.deposit_amount,
    };
    if state.total_amount < state.reclaimable_threshold {
        // TODO : transfer 50% to the community fund
        cw20_msg = Cw20ExecuteMsg::Burn {
            amount: state.deposit_amount,
        };
    }
    state.deposit_reclaimed = true;
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "try_finish_poll")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.token_contract.to_string(),
            msg: to_binary(&cw20_msg)?,
            funds: vec![],
        })))
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
                    denom: DENOM.to_string(),
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
            "cannot claim rewards. current status: {}",
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

    REWARDS.remove(deps.storage, &addr);

    Ok(Response::new()
        .add_attribute("method", "try_claim")
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: addr.to_string(),
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: value,
            }],
        })))
}

pub fn try_reclaim_deposit(deps: DepsMut) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;
    if state.deposit_reclaimed {
        return Err(StdError::generic_err("Already reclaimed".to_string()));
    }

    if state.total_amount < state.reclaimable_threshold {
        return Err(StdError::generic_err("Not enough total amount".to_string()));
    }

    state.deposit_reclaimed = true;
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "try_reclaim_deposit")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.token_contract.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: state.generator.to_string(),
                amount: state.deposit_amount,
            })?,
            funds: vec![],
        })))
}

pub fn try_reset_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_name: String,
    bet_end_time: u64,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;
    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can reset the poll",
        ));
    }
    let bet_status = state.status.clone();
    if bet_status != BetStatus::Reward && bet_status != BetStatus::Closed {
        return Err(StdError::generic_err(
            "You can't reset the poll until the poll ends",
        ));
    }

    // To return all the rewards
    let mut reward_msgs: Vec<CosmosMsg> = vec![];
    REWARDS
        .range(deps.storage, None, None, Order::Ascending)
        .for_each(|item| {
            if let Ok((addr, reward)) = item {
                let addr = str::from_utf8(&addr).unwrap_or("").to_string();
                if addr != String::default() {
                    reward_msgs.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: addr,
                        amount: vec![Coin {
                            denom: DENOM.to_string(),
                            amount: reward,
                        }],
                    }));
                }
            }
        });

    // Withdrawal
    let contract_balance = deps.querier.query_balance(&env.contract.address, DENOM)?;
    let withdrawal_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: state.owner.to_string(),
        amount: vec![Coin {
            denom: DENOM.to_string(),
            amount: contract_balance.amount,
        }],
    });

    // Clear all the states
    let keys: Vec<_> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(k, _)| k)
        .collect();
    for k in keys {
        deps.storage.remove(&k);
    }

    state.status = BetStatus::Voting;
    state.poll_name = poll_name;
    state.bet_end_time = bet_end_time;
    state.total_amount = Uint128::zero();

    STATE.save(deps.storage, &state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "try_reset_poll")
        .add_messages(reward_msgs)
        .add_message(withdrawal_msg))
}

pub fn try_transfer_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;
    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can transfer the ownership",
        ));
    }
    state.owner = deps.api.addr_validate(&new_owner)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "try_transfer_owner"))
}

pub fn try_set_minimum_bet(deps: DepsMut, info: MessageInfo, amount: u128) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;
    if info.sender != state.owner {
        return Err(StdError::generic_err(
            "only the original owner can set the minimum bet amount",
        ));
    }
    state.minimum_bet = Uint128::from(amount);
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "try_set_minimun_amount"))
}
