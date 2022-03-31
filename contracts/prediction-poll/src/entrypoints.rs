use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BetStatus, State, STATE};
use crate::{executions, queries};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:prediction-poll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO : set proper default values
const DEFAULT_MINIMUM_BET: Uint128 = Uint128::new(1_000);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        generator: msg.generator,
        token_contract: msg.token_contract,
        deposit_amount: msg.deposit_amount,
        deposit_reclaimed: false,
        reclaimable_threshold: msg.reclaimable_threshold,
        status: BetStatus::Created,
        poll_name: msg.poll_name,
        start_time: msg.start_time,
        bet_end_time: msg.bet_end_time,
        total_amount: Uint128::new(0),
        minimum_bet: DEFAULT_MINIMUM_BET,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

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
        ExecuteMsg::Bet { side } => executions::try_bet(deps, _env, info, side),
        ExecuteMsg::FinishPoll { winner } => executions::try_finish_poll(deps, _env, info, winner),
        ExecuteMsg::RevertPoll {} => executions::try_revert_poll(deps, info),
        ExecuteMsg::Claim {} => executions::try_claim(deps, info),
        ExecuteMsg::ResetPoll {
            poll_name,
            start_time,
            bet_end_time,
        } => executions::try_reset_poll(deps, _env, info, poll_name, start_time, bet_end_time),
        ExecuteMsg::ReclaimDeposit {} => executions::try_reclaim_deposit(deps),
        ExecuteMsg::TransferOwner { new_owner } => {
            executions::try_transfer_owner(deps, info, new_owner)
        }
        ExecuteMsg::SetMinimumBet { amount } => executions::try_set_minimum_bet(deps, info, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::BetStatus {} => to_binary(&queries::query_bet_status(deps, _env)?),
        QueryMsg::BetLive {} => to_binary(&queries::query_bet_live(deps, _env)?),
        QueryMsg::RewardLive {} => to_binary(&queries::query_reward_live(deps, _env)?),
        QueryMsg::UserBet { address, side } => {
            to_binary(&queries::query_user_bet(deps, address, side)?)
        }
        QueryMsg::UserRewards { address } => {
            to_binary(&queries::query_user_rewards(deps, address)?)
        }
    }
}
