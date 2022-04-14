#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CanonicalAddr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::staking::{query_staker, withdraw_voting_tokens};
use crate::state::{config_store, state_store, Config, State};
use crate::validators::{validate_poll_period, validate_quorum, validate_threshold};

pub(crate) const MAX_QUORUM: Decimal = Decimal::one();
pub(crate) const MAX_THRESHOLD: Decimal = Decimal::one();
pub(crate) const MIN_TITLE_LENGTH: usize = 4;
pub(crate) const MAX_TITLE_LENGTH: usize = 64;
pub(crate) const MIN_DESC_LENGTH: usize = 4;
pub(crate) const MAX_DESC_LENGTH: usize = 1024;
pub(crate) const MIN_LINK_LENGTH: usize = 12;
pub(crate) const MAX_LINK_LENGTH: usize = 128;

use crate::executions;
use crate::queries;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_quorum(msg.quorum)?;
    validate_threshold(msg.threshold)?;
    validate_poll_period(msg.timelock_period, msg.expiration_period)?;

    let config = Config {
        pollterra_token: CanonicalAddr::from(vec![]),
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: msg.expiration_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };

    let state = State {
        contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
    };
    config_store(deps.storage).save(&config)?;
    state_store(deps.storage).save(&state)?;

    Ok(Response::default())
}

// Routers; here is a separate router which handles Execution of functions on the contract or performs a contract Query
// Each router function defines a number of handlers using Rust's pattern matching to
// designated how each ExecutionMsg or QueryMsg will be handled.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Handle 'payable' functionalities
        ExecuteMsg::Receive(msg) => executions::receive_cw20(deps, _env, info, msg),
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => executions::cast_vote(deps, _env, info, poll_id, vote, amount),
        // Mark a poll as ended
        ExecuteMsg::EndPoll { poll_id } => executions::end_poll(deps, _env, poll_id),
        // Execute the associated messages of a passed poll
        ExecuteMsg::ExecutePoll { poll_id } => executions::execute_poll(deps, _env, poll_id),
        ExecuteMsg::ExpirePoll { poll_id } => executions::expire_poll(deps, _env, poll_id),
        ExecuteMsg::RegisterContracts { pollterra_token } => {
            executions::register_contracts(deps, pollterra_token)
        }
        ExecuteMsg::SnapshotPoll { poll_id } => executions::snapshot_poll(deps, _env, poll_id),
        ExecuteMsg::WithdrawVotingTokens { amount } => withdraw_voting_tokens(deps, info, amount),
        ExecuteMsg::UpdateConfig {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        } => executions::update_config(
            deps,
            info,
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&queries::query_config(deps)?)?),
        QueryMsg::State {} => Ok(to_binary(&queries::query_state(deps)?)?),
        QueryMsg::Staker { address } => Ok(to_binary(&query_staker(deps, address)?)?),
        QueryMsg::Poll { poll_id } => Ok(to_binary(&queries::query_poll(deps, poll_id)?)?),
        QueryMsg::Polls {
            filter,
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&queries::query_polls(
            deps,
            filter,
            start_after,
            limit,
            order_by,
        )?)?),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&queries::query_voters(
            deps,
            poll_id,
            start_after,
            limit,
            order_by,
        )?)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
