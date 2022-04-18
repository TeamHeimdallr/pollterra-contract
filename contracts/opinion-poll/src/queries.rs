use cosmwasm_std::{Deps, Env, StdResult, Timestamp};

use messages::opinion_poll::query_msgs::{
    ConfigResponse, PollStatusResponse, StateResponse, UserVoteResponse, VoteCountResponse,
    VoteLiveResponse,
};
use messages::opinion_poll::state::{read_config, read_state, SIDES, VOTES};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;
    Ok(config)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = read_state(deps.storage)?;
    Ok(state)
}

pub fn query_poll_status(deps: Deps) -> StdResult<PollStatusResponse> {
    let status = read_state(deps.storage)?.status;
    Ok(PollStatusResponse { status })
}

pub fn query_vote_live(deps: Deps, env: Env) -> StdResult<VoteLiveResponse> {
    let config = read_config(deps.storage)?;
    let vote_live = env.block.time < Timestamp::from_seconds(config.end_time);

    Ok(VoteLiveResponse { vote_live })
}

pub fn query_vote_count(deps: Deps, side: u64) -> StdResult<VoteCountResponse> {
    let count = (SIDES.may_load(deps.storage, &side.to_be_bytes())?).unwrap_or(0);

    Ok(VoteCountResponse { count })
}

pub fn query_user_vote(deps: Deps, address: String) -> StdResult<UserVoteResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let side = VOTES.may_load(deps.storage, &addr)?;

    Ok(UserVoteResponse { side })
}
