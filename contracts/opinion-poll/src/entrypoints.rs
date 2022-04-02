use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{store_config, store_state, BetStatus, Config, State};
use crate::{executions, queries};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:opinion-poll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: info.sender.clone(),
        generator: msg.generator,
        token_contract: msg.token_contract,
        reclaimable_threshold: msg.reclaimable_threshold,
        poll_name: msg.poll_name,
        bet_end_time: msg.bet_end_time,
        resolution_time: msg.resolution_time,
        // config for prediction poll. not used here.
        minimum_bet_amount: Uint128::zero(),
        tax_percentage: Decimal::zero(),
    };
    let state = State {
        deposit_amount: msg.deposit_amount,
        deposit_reclaimed: false,
        status: BetStatus::Voting,
        total_amount: Uint128::new(0),
        winning_side: None,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    store_state(deps.storage, &state)?;
    store_config(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("deposit_amount", state.deposit_amount))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Vote { side } => executions::vote(deps, _env, info, side),
        ExecuteMsg::FinishPoll {} => executions::finish_poll(deps, _env, info),
        ExecuteMsg::ReclaimDeposit {} => executions::reclaim_deposit(deps),
        ExecuteMsg::TransferOwner { new_owner } => {
            executions::transfer_owner(deps, info, new_owner)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::PollStatus {} => to_binary(&queries::query_poll_status(deps)?),
        QueryMsg::VoteLive {} => to_binary(&queries::query_vote_live(deps, _env)?),
        QueryMsg::VoteCount { side } => to_binary(&queries::query_vote_count(deps, side)?),
        QueryMsg::UserVote { address } => to_binary(&queries::query_user_vote(deps, address)?),
    }
}