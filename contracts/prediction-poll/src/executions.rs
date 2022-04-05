use crate::error::ContractError;
use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use std::str;

use crate::state::{
    read_config, read_state, store_config, store_state, BetStatus, BETS, REWARDS,
    SIDE_TOTAL_AMOUNT, STATE, USER_TOTAL_AMOUNT,
};
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:prediction-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DENOM: &str = "uusd";

pub fn try_bet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    side: u64,
) -> Result<Response, ContractError> {
    let addr = info.sender.clone();

    let config = read_config(deps.storage)?;

    // current block time is less than start time or larger than bet end time
    if env.block.time >= Timestamp::from_seconds(config.bet_end_time) {
        return Err(ContractError::BetIsNotLive(
            env.block.time,
            config.bet_end_time,
        ));
    }

    // Check if some funds are sent
    let sent = match info.funds.len() {
        0 => Err(ContractError::EmptyFunds {}),
        1 => {
            if info.funds[0].denom == DENOM {
                Ok(info.funds[0].amount)
            } else {
                Err(ContractError::OnlyUstAvailable {})
            }
        }
        _ => Err(ContractError::OnlyUstAvailable {}),
    }?;

    // sent 0 ust case
    if sent.is_zero() {
        return Err(ContractError::EmptyFunds {});
    }

    if sent < config.minimum_bet_amount {
        return Err(ContractError::LessThanMinimumBetAmount(
            config.minimum_bet_amount,
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
    let mut state = read_state(deps.storage)?;
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
    winner: u64,
) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    // only contract's owner can finish poll
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // already finished
    if state.status == BetStatus::Closed || state.status == BetStatus::Reward {
        return Err(ContractError::AlreadyFinishedPoll {});
    }

    // cannot finish before poll ends
    if env.block.time < Timestamp::from_seconds(config.bet_end_time) {
        return Err(ContractError::FinishBeforeEndTime {});
    }

    let winner_amount = match SIDE_TOTAL_AMOUNT.may_load(deps.storage, &winner.to_be_bytes())? {
        Some(value) => value,
        None => Uint128::new(0),
    };

    let odds = Decimal::from_ratio(state.total_amount, winner_amount);

    // iterate over them all
    let all: StdResult<Vec<_>> = BETS
        .prefix(&winner.to_be_bytes())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, reward) = item?;
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
    }

    // Save the new state
    state.status = BetStatus::Reward;
    state.winning_side = Some(vec![winner]);

    let mut cw20_msg = Cw20ExecuteMsg::Transfer {
        recipient: config.generator.to_string(),
        amount: state.deposit_amount,
    };
    if state.total_amount < config.reclaimable_threshold {
        // TODO : transfer 50% to the community fund
        cw20_msg = Cw20ExecuteMsg::Burn {
            amount: state.deposit_amount,
        };
    }
    state.deposit_reclaimed = true;
    store_state(deps.storage, &state)?;

    // TODO : get n% of rewards as a tax here or in try_bet
    Ok(Response::new()
        .add_attribute("method", "try_finish_poll")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.token_contract,
            msg: to_binary(&cw20_msg)?,
            funds: vec![],
        })))
}

pub fn try_revert_poll(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    // TODO: make sure all of users didn't claim rewards

    // only contract's owner can revert
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // already reverted
    if state.status == BetStatus::Closed {
        return Err(ContractError::AlreadyReverted {});
    }

    let mut msgs: Vec<CosmosMsg> = vec![];

    // iterate over them all
    let _all: StdResult<Vec<_>> = USER_TOTAL_AMOUNT
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, spent) = item?;
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

    // TODO? USER_TOTAL_AMOUNT reset? not necessary

    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "try_revert_poll")
        .add_messages(msgs))
}

pub fn try_claim(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let addr = info.sender;
    let state = read_state(deps.storage)?;

    if state.status != BetStatus::Reward {
        return Err(ContractError::CannotClaimRewards(state.status));
    }

    // REWARDS State load
    let value = match REWARDS.may_load(deps.storage, &addr)? {
        None => Uint128::zero(),
        Some(amount) => amount,
    };

    if value == Uint128::zero() {
        return Err(ContractError::EmptyRewards {});
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

pub fn try_reclaim_deposit(deps: DepsMut) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;
    if state.deposit_reclaimed {
        return Err(ContractError::AlreadyReclaimed {});
    }

    if state.total_amount < config.reclaimable_threshold {
        return Err(ContractError::InsufficientReclaimableThreshold(
            state.total_amount,
            config.reclaimable_threshold,
        ));
    }

    state.deposit_reclaimed = true;
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "try_reclaim_deposit")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.token_contract.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.generator.to_string(),
                amount: state.deposit_amount,
            })?,
            funds: vec![],
        })))
}

// TODO : remove this and create a function named wrap_up_poll
pub fn try_reset_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _poll_name: String,
    _bet_end_time: u64,
) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let bet_status = state.status.clone();
    if bet_status != BetStatus::Reward && bet_status != BetStatus::Closed {
        return Err(ContractError::ResetBeforeClosed {});
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
        to_address: config.owner.to_string(),
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
    state.total_amount = Uint128::zero();

    STATE.save(deps.storage, &state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "try_reset_poll")
        .add_messages(reward_msgs)
        .add_message(withdrawal_msg))
}

// TODO : create update_config function
pub fn try_transfer_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    config.owner = deps.api.addr_validate(&new_owner)?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "try_transfer_owner"))
}

pub fn try_set_minimun_bet_amount(
    deps: DepsMut,
    info: MessageInfo,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    config.minimum_bet_amount = Uint128::from(amount);
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "try_set_minimun_amount"))
}
