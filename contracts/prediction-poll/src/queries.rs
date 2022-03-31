use cosmwasm_std::{Deps, Env, StdResult, Timestamp, Uint128};

use crate::msg::{
    BetLiveResponse, BetStatusResponse, ConfigResponse, RewardLiveResponse, UserBetResponse,
    UserRewardsResponse,
};
use crate::state::{read_state, BetStatus, BETS, REWARDS};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_state(deps.storage)?;
    Ok(state)
}

pub fn query_bet_status(deps: Deps, env: Env) -> StdResult<BetStatusResponse> {
    let state = read_state(deps.storage)?;

    let status = if state.status == BetStatus::Closed {
        BetStatus::Closed
    } else if env.block.time < Timestamp::from_seconds(state.start_time) {
        BetStatus::Created
    } else if env.block.time < Timestamp::from_seconds(state.bet_end_time) {
        BetStatus::Betting
    } else if state.status == BetStatus::Reward {
        BetStatus::Reward
    } else {
        BetStatus::BetHold
    };

    Ok(BetStatusResponse { status })
}

pub fn query_bet_live(deps: Deps, env: Env) -> StdResult<BetLiveResponse> {
    let state = read_state(deps.storage)?;
    let bet_live = env.block.time > Timestamp::from_seconds(state.start_time)
        && env.block.time < Timestamp::from_seconds(state.bet_end_time);

    Ok(BetLiveResponse { bet_live })
}

pub fn query_reward_live(deps: Deps, env: Env) -> StdResult<RewardLiveResponse> {
    let reward_live = query_bet_status(deps, env)?.status == BetStatus::Reward;
    Ok(RewardLiveResponse { reward_live })
}

pub fn query_user_bet(deps: Deps, address: String, side: u8) -> StdResult<UserBetResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let amount = match BETS.may_load(deps.storage, (&side.to_be_bytes(), &addr))? {
        None => Uint128::new(0),
        Some(amount) => amount,
    };

    Ok(UserBetResponse { amount })
}

pub fn query_user_rewards(deps: Deps, address: String) -> StdResult<UserRewardsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let reward = match REWARDS.may_load(deps.storage, &addr)? {
        None => Uint128::new(0),
        Some(amount) => amount,
    };

    Ok(UserRewardsResponse { reward })
}
