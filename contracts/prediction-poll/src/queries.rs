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

    // bet_live can be incorrect if "bet" execution is not called. therefore use block time
    let bet_status = if state.status == BetStatus::Closed {
        BetStatus::Closed
    } else if env.block.time < Timestamp::from_seconds(state.start_time) {
        BetStatus::Created
    } else if env.block.time < Timestamp::from_seconds(state.bet_end_time) {
        BetStatus::Betting
    } else if state.reward_live {
        BetStatus::Reward
    } else {
        BetStatus::BetHold
    };

    Ok(BetStatusResponse { status: bet_status })
}

pub fn query_bet_live(deps: Deps, env: Env) -> StdResult<BetLiveResponse> {
    let is_bet_live = query_bet_status(deps, env)?.status == BetStatus::Betting;

    Ok(BetLiveResponse {
        bet_live: is_bet_live,
    })
}

pub fn query_reward_live(deps: Deps) -> StdResult<RewardLiveResponse> {
    let state = read_state(deps.storage)?;
    Ok(RewardLiveResponse {
        reward_live: state.reward_live,
    })
}

pub fn query_user_bet(deps: Deps, address: String, side: u8) -> StdResult<UserBetResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let value = match BETS.may_load(deps.storage, (&side.to_be_bytes(), &addr))? {
        None => Uint128::new(0),
        Some(amount) => amount,
    };

    Ok(UserBetResponse { amount: value })
}

pub fn query_user_rewards(deps: Deps, address: String) -> StdResult<UserRewardsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let value = match REWARDS.may_load(deps.storage, &addr)? {
        None => Uint128::new(0),
        Some(amount) => amount,
    };

    Ok(UserRewardsResponse { reward: value })
}
