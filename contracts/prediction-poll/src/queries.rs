use cosmwasm_std::{Deps, Env, StdResult, Timestamp, Uint128};

use messages::prediction_poll::query_msgs::{
    BetLiveResponse, BetStatusResponse, ConfigResponse, RewardLiveResponse, StateResponse,
    UserBetResponse, UserRewardsResponse,
};
use messages::prediction_poll::state::{read_config, read_state, BetStatus, BETS, REWARDS};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;
    Ok(config)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = read_state(deps.storage)?;
    Ok(state)
}

pub fn query_bet_status(deps: Deps) -> StdResult<BetStatusResponse> {
    let status = read_state(deps.storage)?.status;
    Ok(BetStatusResponse { status })
}

pub fn query_bet_live(deps: Deps, env: Env) -> StdResult<BetLiveResponse> {
    let config = read_config(deps.storage)?;
    let state = read_state(deps.storage)?;
    let bet_live = env.block.time < Timestamp::from_seconds(config.end_time)
        && state.status == BetStatus::Voting;

    Ok(BetLiveResponse { bet_live })
}

pub fn query_reward_live(deps: Deps) -> StdResult<RewardLiveResponse> {
    let reward_live = query_bet_status(deps)?.status == BetStatus::Reward;
    Ok(RewardLiveResponse { reward_live })
}

pub fn query_user_bet(deps: Deps, address: String, side: u64) -> StdResult<UserBetResponse> {
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
