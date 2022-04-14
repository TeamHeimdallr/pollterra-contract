use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::{executions, queries};
use messages::prediction_poll::execute_msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use messages::prediction_poll::query_msgs::QueryMsg;
use messages::prediction_poll::state::{store_config, store_state, BetStatus, Config, State};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:prediction-poll";
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
        poll_type: msg.poll_type,
        bet_end_time: msg.bet_end_time,
        resolution_time: msg.resolution_time,
        minimum_bet_amount: msg.minimum_bet_amount.unwrap(),
        tax_percentage: msg.tax_percentage.unwrap(),
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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bet { side } => executions::try_bet(deps, _env, info, side),
        ExecuteMsg::FinishPoll { winner } => executions::try_finish_poll(deps, _env, info, winner),
        ExecuteMsg::RevertPoll {} => executions::try_revert_poll(deps, info),
        ExecuteMsg::Claim {} => executions::try_claim(deps, info),
        ExecuteMsg::ReclaimDeposit {} => executions::try_reclaim_deposit(deps),
        ExecuteMsg::TransferOwner { new_owner } => {
            executions::try_transfer_owner(deps, info, new_owner)
        }
        ExecuteMsg::SetMinimumBet { amount } => {
            executions::try_set_minimun_bet_amount(deps, info, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::BetStatus {} => to_binary(&queries::query_bet_status(deps)?),
        QueryMsg::BetLive {} => to_binary(&queries::query_bet_live(deps, _env)?),
        QueryMsg::RewardLive {} => to_binary(&queries::query_reward_live(deps)?),
        QueryMsg::UserBet { address, side } => {
            to_binary(&queries::query_user_bet(deps, address, side)?)
        }
        QueryMsg::UserRewards { address } => {
            to_binary(&queries::query_user_rewards(deps, address)?)
        }
    }
}
