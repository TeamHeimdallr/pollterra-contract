#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, StdResult};

use crate::error::ContractError;
use crate::state::{
    config_read, poll_read, read_poll_voters, read_polls, state_read, Config, ConfigResponse,
    OrderBy, Poll, PollExecuteMsg, PollResponse, PollStatus, PollsResponse, State, StateResponse,
    VotersResponse, VotersResponseItem,
};

/// query_config allows for the query of the currently set configuration values
/// which influence Polls such as the quorum needed and the minimum voting peroid before a poll can be ended
pub fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        pollterra_token: deps.api.addr_humanize(&config.pollterra_token)?.to_string(),
        quorum: config.quorum,
        threshold: config.threshold,
        voting_period: config.voting_period,
        timelock_period: config.timelock_period,
        expiration_period: config.expiration_period,
        proposal_deposit: config.proposal_deposit,
        snapshot_period: config.snapshot_period,
    })
}

/// query_state allows for the query of dynamic state values such as the poll count and how much has been deposited
pub fn query_state(deps: Deps) -> Result<StateResponse, ContractError> {
    let state: State = state_read(deps.storage).load()?;
    Ok(StateResponse {
        poll_count: state.poll_count,
        total_share: state.total_share,
        total_deposit: state.total_deposit,
    })
}

/// query_poll allows for the query of a given poll by supplying its poll_id
pub fn query_poll(deps: Deps, poll_id: u64) -> Result<PollResponse, ContractError> {
    let poll = match poll_read(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let mut data_list: Vec<PollExecuteMsg> = vec![];

    Ok(PollResponse {
        id: poll.id,
        creator: deps.api.addr_humanize(&poll.creator)?.to_string(),
        status: poll.status,
        end_height: poll.end_height,
        title: poll.title,
        description: poll.description,
        link: poll.link,
        deposit_amount: poll.deposit_amount,
        execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
            for msg in exe_msgs {
                let execute_data = PollExecuteMsg {
                    order: msg.order,
                    contract: deps.api.addr_humanize(&msg.contract)?.to_string(),
                    msg: msg.msg,
                };
                data_list.push(execute_data)
            }
            Some(data_list)
        } else {
            None
        },
        yes_votes: poll.yes_votes,
        no_votes: poll.no_votes,
        staked_amount: poll.staked_amount,
        total_balance_at_end_poll: poll.total_balance_at_end_poll,
    })
}

pub fn query_polls(
    deps: Deps,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<PollsResponse, ContractError> {
    let polls = read_polls(deps.storage, filter, start_after, limit, order_by)?;

    let poll_responses: StdResult<Vec<PollResponse>> = polls
        .iter()
        .map(|poll| {
            Ok(PollResponse {
                id: poll.id,
                creator: deps.api.addr_humanize(&poll.creator)?.to_string(),
                status: poll.status.clone(),
                end_height: poll.end_height,
                title: poll.title.to_string(),
                description: poll.description.to_string(),
                link: poll.link.clone(),
                deposit_amount: poll.deposit_amount,
                execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
                    let mut data_list: Vec<PollExecuteMsg> = vec![];

                    for msg in exe_msgs {
                        let execute_data = PollExecuteMsg {
                            order: msg.order,
                            contract: deps.api.addr_humanize(&msg.contract)?.to_string(),
                            msg: msg.msg,
                        };
                        data_list.push(execute_data)
                    }
                    Some(data_list)
                } else {
                    None
                },
                yes_votes: poll.yes_votes,
                no_votes: poll.no_votes,
                staked_amount: poll.staked_amount,
                total_balance_at_end_poll: poll.total_balance_at_end_poll,
            })
        })
        .collect();

    Ok(PollsResponse {
        polls: poll_responses?,
    })
}

pub fn query_voters(
    deps: Deps,
    poll_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<VotersResponse, ContractError> {
    let poll: Poll = match poll_read(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let voters = if poll.status != PollStatus::InProgress {
        vec![]
    } else if let Some(start_after) = start_after {
        read_poll_voters(
            deps.storage,
            poll_id,
            Some(deps.api.addr_canonicalize(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        read_poll_voters(deps.storage, poll_id, None, limit, order_by)?
    };

    let voters_response: StdResult<Vec<VotersResponseItem>> = voters
        .iter()
        .map(|voter_info| {
            Ok(VotersResponseItem {
                voter: deps.api.addr_humanize(&voter_info.0)?.to_string(),
                vote: voter_info.1.vote.clone(),
                balance: voter_info.1.balance,
            })
        })
        .collect();

    Ok(VotersResponse {
        voters: voters_response?,
    })
}
