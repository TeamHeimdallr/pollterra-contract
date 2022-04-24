use crate::error::ContractError;
use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Timestamp, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use std::str;

use messages::prediction_poll::state::{
    read_config, read_state, store_config, store_state, BetStatus, BETS, REWARDS,
    SIDE_TOTAL_AMOUNT, USER_TOTAL_AMOUNT,
};

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
    if env.block.time >= Timestamp::from_seconds(config.end_time) {
        return Err(ContractError::BetIsNotLive(env.block.time, config.end_time));
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

    if side >= config.num_side {
        return Err(ContractError::SideOutOfRange(config.num_side));
    }

    let update_action = |exists: Option<Uint128>| -> StdResult<Uint128> {
        match exists {
            Some(bet) => Ok(bet + sent),
            None => Ok(sent),
        }
    };

    // add bet amount to BETS state (accumulate single side)
    BETS.update(deps.storage, (&side.to_be_bytes(), &addr), update_action)?;

    // add bet amount to USER_TOTAL_AMOUNT state (accumulate both side)
    USER_TOTAL_AMOUNT.update(deps.storage, &addr, update_action)?;

    // add bet amount to SIDE_TOTAL_AMOUNT state (accumulate single side, every user)
    SIDE_TOTAL_AMOUNT.update(deps.storage, &side.to_be_bytes(), update_action)?;

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
    forced: bool,
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
    if !forced && env.block.time < Timestamp::from_seconds(config.resolution_time) {
        return Err(ContractError::FinishBeforeEndTime {});
    }

    if winner >= config.num_side {
        return Err(ContractError::SideOutOfRange(config.num_side));
    }

    let mut response = Response::new().add_attribute("method", "try_finish_poll");

    let winner_amount = match SIDE_TOTAL_AMOUNT.may_load(deps.storage, &winner.to_be_bytes())? {
        Some(value) => value,
        None => Uint128::new(0),
    };

    let mut total_rewards = Uint128::zero();

    // Give it all back when no winner
    if winner_amount.is_zero() {
        let all: StdResult<Vec<_>> = BETS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item| {
                let (addr, bet_amount) = item?;
                Ok((addr, bet_amount))
            })
            .collect();

        for (addr, bet_amount) in all?.iter() {
            REWARDS.update(
                deps.storage,
                &deps.api.addr_validate(str::from_utf8(addr)?)?,
                |exists| -> StdResult<Uint128> {
                    match exists {
                        Some(exist) => Ok(exist + bet_amount),
                        None => Ok(*bet_amount),
                    }
                },
            )?;
            total_rewards += bet_amount;
        }
    } else {
        let total_amount_deducted = (state.total_amount - winner_amount)
            * (Decimal::percent(100_u64) - config.tax_percentage)
            + winner_amount;
        let odds = Decimal::from_ratio(total_amount_deducted, winner_amount);

        let all: StdResult<Vec<_>> = BETS
            .prefix(&winner.to_be_bytes())
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item| {
                let (addr, bet_amount) = item?;
                Ok((addr, bet_amount))
            })
            .collect();

        for (addr, bet_amount) in all?.iter() {
            let reward = *bet_amount * odds;
            REWARDS.update(
                deps.storage,
                &deps.api.addr_validate(str::from_utf8(addr)?)?,
                |_exists| -> StdResult<Uint128> { Ok(reward) },
            )?;
            total_rewards += reward;
        }
    }

    // transfer remain amount to contract owner
    let contract_balance = deps
        .querier
        .query_balance(env.contract.address, DENOM.to_string())?;
    let transfer_amount = contract_balance.amount - total_rewards;
    if !transfer_amount.is_zero() {
        let transfer_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
            to_address: config.owner.to_string(),
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: transfer_amount,
            }],
        });
        response = response.add_message(transfer_msg);
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

    Ok(response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
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
