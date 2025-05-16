use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin as CosmosCoin, CosmosMsg, Deps,
    DepsMut, Empty, Env, MessageInfo, Order, Response, StakingMsg, StdResult, Uint128,
};
use cw2::set_contract_version;
use std::str::FromStr;

// Coreum imports

use crate::error::ContractError;
use crate::msg::{
    AccumulatedRewardsAtUndelegationResponse, AccumulatedRewardsResponse, BonusRewardsResponse,
    ClaimInfo, ClaimsResponse, CurrentStateResponse, DelegatedAmountResponse, DraftTvlResponse,
    ExecuteMsg, InstantiateMsg, ParticipantInfo, ParticipantsResponse, QueryMsg,
    TicketHoldersResponse, TicketsSoldResponse, TotalBurnedResponse, UserTicketsResponse,
    UserWinChanceResponse, WinnerResponse,
};
use crate::state::{
    all_tickets_burned, calculate_win_chance, decrease_ticket_holder, get_draft_tvl,
    increment_tickets_burned, increment_tickets_sold, initialize_storage,
    should_close_ticket_sales, update_claim, update_ticket_holder, Config, DrawState,
    ACCUMALTED_REWARDS_AT_UNDELEGATION, CLAIMS, CONFIG, TICKET_DENOM, TICKET_HOLDERS,
    TOTAL_TICKETS_BURNED, TOTAL_TICKETS_SOLD,
};

use coreum_wasm_sdk::types::cosmos::base::v1beta1::Coin;

use coreum_wasm_sdk::types::coreum::asset::ft::v1::{
    MsgIssue, MsgMint, QueryBalanceRequest, QueryBalanceResponse,
};

// Version info for migration
const CONTRACT_NAME: &str = "coreum-fun";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Constants
const TICKET_PRECISION: u32 = 6;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<Empty>, ContractError> {
    // Step 1: Set contract version for migrations
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Step 2: Validate input parameters
    if msg.total_tickets.is_zero() {
        return Err(ContractError::InvalidTicketAmount {});
    }

    if msg.ticket_price.is_zero() {
        return Err(ContractError::InvalidTicketPrice {});
    }

    // Step 3: Validate validator address
    // deps.api.validator_address(&msg.validator_address)?;

    // Step 4: Initialize config with default values
    let config = Config {
        owner: info.sender.clone(),
        ticket_symbol: msg.ticket_token_symbol.clone(),
        core_denom: msg.core_denom.clone(),
        validator_address: msg.validator_address.clone(),
        total_tickets: msg.total_tickets,
        //in ucore
        ticket_price: msg.ticket_price,
        max_tickets_per_user: msg.max_tickets_per_user,
        draw_state: DrawState::TicketSalesOpen,
        winner: None,
        undelegation_done_timestamp: None,
        accumulated_rewards: Uint128::zero(),
        bonus_rewards: Uint128::zero(),
    };

    // Step 5: Save config and initialize counters
    CONFIG.save(deps.storage, &config)?;
    initialize_storage(deps.storage)?;

    // Step 6: Create the TICKET smart token (first time setup)
    let issue_token_msg = MsgIssue {
        issuer: env.contract.address.to_string(),
        symbol: msg.ticket_token_symbol.clone(),
        subunit: format!("u{}", msg.ticket_token_symbol.to_lowercase()),
        precision: TICKET_PRECISION, // TODO: check if we go with 0 or 6
        initial_amount: "0".to_string(),
        description: "Draft tickets for Coreum No-Loss Draft on coreum.fun".to_string(),
        //Minting & Burning is enabled
        features: vec![0 as i32, 1 as i32],
        burn_rate: "0".to_string(),
        send_commission_rate: "0".to_string(),
        uri: "https://coreum.fun".to_string(),
        uri_hash: "".to_string(),
        dex_settings: None,
        extension_settings: None,
    };

    //Step 7 construct the denom and save it in the contract state
    let denom = format!(
        "u{}-{}",
        msg.ticket_token_symbol.to_lowercase(),
        env.contract.address
    );

    TICKET_DENOM.save(deps.storage, &denom)?;

    // Step 7: Return success response
    Ok(Response::new()
        .add_message(CosmosMsg::Any(issue_token_msg.to_any()))
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.to_string())
        .add_attribute("ticket_token_symbol", msg.ticket_token_symbol)
        .add_attribute("validator_address", msg.validator_address)
        .add_attribute("total_tickets", msg.total_tickets.to_string())
        .add_attribute("ticket_price", msg.ticket_price.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyTicket { number_of_tickets } => {
            execute_buy_ticket(deps, env, info, number_of_tickets)
        }
        ExecuteMsg::SelectWinnerAndUndelegate { winner_address } => {
            execute_select_winner_and_undelegate(deps, env, info, winner_address)
        }
        ExecuteMsg::SendFundsToWinner {} => execute_send_funds_to_winner(deps, env, info),
        ExecuteMsg::BurnTickets { number_of_tickets } => {
            execute_burn_tickets(deps, env, info, number_of_tickets)
        }
        ExecuteMsg::AddBonusRewardToThePool { amount } => {
            execute_add_bonus_reward(deps, env, info, amount)
        }
        ExecuteMsg::UpdateDrawState { new_state } => {
            execute_update_draw_state(deps, env, info, new_state)
        }
        ExecuteMsg::SendFunds { recipient, amount } => {
            execute_send_funds(deps, info, recipient, amount)
        }
        ExecuteMsg::SetUndelegationTimestamp { timestamp } => {
            execute_set_undelegation_timestamp(deps, env, info, timestamp)
        }
    }
}

pub fn execute_buy_ticket(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    number_of_tickets: Uint128,
) -> Result<Response, ContractError> {
    // Step 1: Verify the COREUM amount sent
    let config = CONFIG.load(deps.storage)?;
    let required_payment = number_of_tickets * config.ticket_price;

    let payment = info
        .funds
        .iter()
        .find(|coin| coin.denom == config.core_denom)
        .ok_or(ContractError::NoFunds {})?;

    if payment.amount < required_payment {
        return Err(ContractError::InsufficientFunds {
            required: required_payment,
            provided: payment.amount,
        });
    }

    // Step 2: Verify that draft is still open for ticket sales
    if config.draw_state != DrawState::TicketSalesOpen {
        return Err(ContractError::TicketSalesClosed {});
    }

    // Step 3: Verify that some tickets are left
    let total_sold = TOTAL_TICKETS_SOLD.load(deps.storage)?;
    if total_sold + number_of_tickets > config.total_tickets {
        return Err(ContractError::NotEnoughTicketsLeft {
            requested: number_of_tickets,
            available: config.total_tickets - total_sold,
        });
    }

    // Step 4: Verify the user have less tickets than the max allowed (counting the new purchase)
    let balance = query_ticket_balance(deps.as_ref(), info.sender.to_string())?;
    let user_tickets: Uint128 =
        Uint128::from_str(&balance.balance)? / Uint128::from(10u128).pow(TICKET_PRECISION);
    let max_tickets_per_user = CONFIG.load(deps.storage)?.max_tickets_per_user;

    if user_tickets + number_of_tickets > max_tickets_per_user {
        return Err(ContractError::MaxTicketsPerUserReached {
            requested: number_of_tickets,
            available: max_tickets_per_user.saturating_sub(user_tickets),
        });
    }

    // Step 5: Stake the COREUM to Coreum Labs validator
    // This is done with every ticket purchase - funds are immediately staked
    let stake_msg = StakingMsg::Delegate {
        validator: config.validator_address.clone(),
        amount: CosmosCoin {
            denom: config.core_denom.clone(),
            amount: required_payment,
        },
    };

    // Step 6: Mint and send the TICKET smart token to the user
    let mint_msg = MsgMint {
        sender: env.contract.address.to_string(),
        coin: Some(Coin {
            denom: TICKET_DENOM.load(deps.storage)?,
            amount: (number_of_tickets * Uint128::from(10u128).pow(TICKET_PRECISION)).to_string(),
        }),
        recipient: info.sender.to_string(),
    };

    // Step 7: Update the contract internal state
    increment_tickets_sold(deps.storage, number_of_tickets)?;
    update_ticket_holder(deps.storage, &info.sender, number_of_tickets)?;

    // Step 8: Check if this was the last ticket - set draw_state=tickets_sold_out_accumulation_in_progress
    let tickets_str = number_of_tickets.to_string();
    let payment_str = required_payment.to_string();
    let mut attrs = vec![
        ("action", "buy_ticket"),
        ("buyer", info.sender.as_str()),
        ("tickets_purchased", &tickets_str),
        ("payment_amount", &payment_str),
    ];

    if should_close_ticket_sales(deps.storage)? {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.draw_state = DrawState::TicketsSoldOutAccumulationInProgress;
            Ok(config)
        })?;
        attrs.push(("ticket_sales", "closed"));
        attrs.push(("new_state", "TicketsSoldOutAccumulationInProgress"));
    }

    // Step 9: Emit an event for the indexer
    // Handled by the attributes in the response

    // Step 10: Return response with all messages and events
    Ok(Response::new()
        .add_message(CosmosMsg::Staking(stake_msg))
        .add_message(CosmosMsg::Any(mint_msg.to_any()))
        .add_attributes(attrs))
}

pub fn execute_select_winner_and_undelegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    winner_address: String,
) -> Result<Response, ContractError> {
    // Step 1: Receive the winner address
    let winner_addr = deps.api.addr_validate(&winner_address)?;

    // Step 2: Verify the caller is the owner
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Step 3: Verify the draw is in the correct state
    if config.draw_state != DrawState::TicketsSoldOutAccumulationInProgress {
        return Err(ContractError::InvalidDrawState {
            expected: DrawState::TicketsSoldOutAccumulationInProgress,
            actual: config.draw_state,
        });
    }

    // Step 4: Verify the winner has tickets
    let winner_tickets = TICKET_HOLDERS
        .may_load(deps.storage, &winner_addr)?
        .unwrap_or(Uint128::zero());

    if winner_tickets.is_zero() {
        return Err(ContractError::NoTicketsForAddress {});
    }

    // Step 5: Query accumulated rewards
    let accumulated_rewards = query_accumulated_rewards(deps.as_ref(), &env)?;

    ACCUMALTED_REWARDS_AT_UNDELEGATION
        .save(deps.storage, &accumulated_rewards.accumulated_rewards)?;

    let total_rewards = accumulated_rewards.accumulated_rewards + config.bonus_rewards;

    // Step 6: Set the winner address in the contract state
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.winner = Some(winner_addr.clone());
        config.accumulated_rewards = accumulated_rewards.accumulated_rewards;
        config.draw_state = DrawState::WinnerSelectedUndelegationInProcess;
        Ok(config)
    })?;

    // Step 7: Start the undelegation process for all the tokens
    let delegation = deps.querier.query_delegation(
        env.contract.address.to_string(),
        config.validator_address.clone(),
    )?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let Some(delegation) = delegation {
        let undelegate_msg = StakingMsg::Undelegate {
            validator: config.validator_address.clone(),
            amount: CosmosCoin {
                denom: config.core_denom.clone(),
                amount: delegation.amount.amount,
            },
        };
        messages.push(CosmosMsg::Staking(undelegate_msg));
    }

    // Step 8: Calculate the timestamp at which the undelegation will be completed
    const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
    const UNDELEGATION_DAYS: u64 = 7;
    let undelegation_period_seconds: u64 = SECONDS_PER_DAY * UNDELEGATION_DAYS;
    let undelegation_done_timestamp = env.block.time.seconds() + undelegation_period_seconds;

    // Step 9: Update the contract state with the future timestamp
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.undelegation_done_timestamp = Some(undelegation_done_timestamp);
        Ok(config)
    })?;

    // Return response with all actions
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "select_winner_and_undelegate"),
        ("winner", &winner_addr.to_string()),
        ("rewards_amount", &total_rewards.to_string()),
        (
            "undelegation_done_timestamp",
            &undelegation_done_timestamp.to_string(),
        ),
        ("new_state", "WinnerSelectedUndelegationInProcess"),
    ]))
}

pub fn execute_send_funds_to_winner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Step 1: Verify the caller is the owner
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Step 2: Verify the draw is in the correct state
    if config.draw_state != DrawState::WinnerSelectedUndelegationInProcess {
        return Err(ContractError::InvalidDrawState {
            expected: DrawState::WinnerSelectedUndelegationInProcess,
            actual: config.draw_state,
        });
    }

    // Step 3: Check if undelegation period is complete
    if let Some(undelegation_timestamp) = config.undelegation_done_timestamp {
        if env.block.time.seconds() < undelegation_timestamp {
            return Err(ContractError::UndelegationPeriodNotCompleted {
                current_timestamp: env.block.time.seconds(),
                undelegation_timestamp,
            });
        }
    }

    // Step 4: Get winner address
    let winner_addr = config.winner.ok_or(ContractError::NoWinnerSelected {})?;

    // Step 5: Calculate total rewards
    let total_rewards = config.accumulated_rewards + config.bonus_rewards;

    // Step 6: Send the rewards to the winner
    let send_rewards_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: winner_addr.to_string(),
        amount: vec![CosmosCoin {
            denom: config.core_denom.clone(),
            amount: total_rewards,
        }],
    });

    // Return response with all actions
    Ok(Response::new()
        .add_message(send_rewards_msg)
        .add_attributes(vec![
            ("action", "send_funds_to_winner"),
            ("winner", &winner_addr.to_string()),
            ("rewards_amount", &total_rewards.to_string()),
        ]))
}

pub fn execute_burn_tickets(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    number_of_tickets: Uint128,
) -> Result<Response, ContractError> {
    // Step 1: Verify the draw is in the correct state
    let config = CONFIG.load(deps.storage)?;
    if config.draw_state != DrawState::WinnerSelectedUndelegationInProcess
        && config.draw_state != DrawState::UndelegationCompletedTokensCanBeBurned
    {
        return Err(ContractError::InvalidDrawState {
            expected: DrawState::UndelegationCompletedTokensCanBeBurned,
            actual: config.draw_state,
        });
    }

    // Check if undelegation period is complete and update state if needed
    if config.draw_state == DrawState::WinnerSelectedUndelegationInProcess {
        if let Some(undelegation_timestamp) = config.undelegation_done_timestamp {
            if env.block.time.seconds() >= undelegation_timestamp {
                CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
                    config.draw_state = DrawState::UndelegationCompletedTokensCanBeBurned;
                    Ok(config)
                })?;
            } else {
                return Err(ContractError::UndelegationPeriodNotCompleted {
                    current_timestamp: env.block.time.seconds(),
                    undelegation_timestamp,
                });
            }
        }
    }

    // Step 2: Verify the user has enough tickets
    // Not going to work because the user sent the tickets in the funds!
    // let balance_before_burn = query_ticket_balance(deps.as_ref(), info.sender.to_string())?;
    // let user_tickets_before_burn = Uint128::from_str(&balance_before_burn.balance)?;
    // println!("user_tickets_before_burn: {}", user_tickets_before_burn);

    // if user_tickets_before_burn < number_of_tickets.pow(TICKET_PRECISION) {
    //     return Err(ContractError::NotEnoughTickets {
    //         requested: number_of_tickets.pow(TICKET_PRECISION),
    //         available: user_tickets_before_burn,
    //     });
    // }

    //Step3: Check if the user sent the correct amount of Ticket in the funds based on the number of tickets they want to burn

    let ticket_denom = TICKET_DENOM.load(deps.storage)?;
    let payment = info
        .funds
        .iter()
        .find(|coin| coin.denom == ticket_denom)
        .ok_or(ContractError::NoFunds {})?;

    if payment.amount < number_of_tickets * Uint128::from(10u128).pow(TICKET_PRECISION) {
        return Err(ContractError::InsufficientFunds {
            required: number_of_tickets * Uint128::from(10u128).pow(TICKET_PRECISION),
            provided: payment.amount,
        });
    }

    //Step 4: The contract now burns the the TICKET tokens
    // let burn_msg = MsgBurn {
    //     sender: env.contract.address.to_string(),
    //     coin: Some(Coin {
    //         denom: TICKET_DENOM.load(deps.storage)?,
    //         amount: number_of_tickets.to_string(),
    //     }),
    // };

    // Step 4: Calculate the refund amount (original investment)
    //We use the users_tickets instead of the requested number of tickets
    let refund_amount: Uint128 = number_of_tickets * config.ticket_price;

    // Step 5: Send back the COREUM to the user
    let send_refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![CosmosCoin {
            denom: config.core_denom.clone(),
            amount: refund_amount,
        }],
    });

    // Step 6: Update internal state - user tickets and total burned
    decrease_ticket_holder(deps.storage, &info.sender, number_of_tickets)?;
    increment_tickets_burned(deps.storage, number_of_tickets)?;
    update_claim(deps.storage, &info.sender, refund_amount)?;

    // Step 7: Check if all tickets have been burned, set draw_state=draw_finished if so
    let tickets_str = number_of_tickets.to_string();
    let refund_str = refund_amount.to_string();
    let mut attrs = vec![
        ("action", "burn_tickets"),
        ("burner", info.sender.as_str()),
        ("tickets_burned", &tickets_str),
        ("refund_amount", &refund_str),
    ];

    if all_tickets_burned(deps.storage)? {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.draw_state = DrawState::DrawFinished;
            Ok(config)
        })?;
        attrs.push(("new_state", "DrawFinished"));
    }

    // Return the response with all actions
    Ok(Response::new()
        // .add_message(CosmosMsg::Any(burn_msg.to_any()))
        .add_message(send_refund_msg)
        .add_attributes(attrs))
}

pub fn execute_add_bonus_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Check if the user sent CORE tokens
    let config = CONFIG.load(deps.storage)?;
    let sent_funds = info
        .funds
        .iter()
        .find(|coin| coin.denom == config.core_denom)
        .ok_or(ContractError::NoFunds {})?;

    // Verify the amount matches
    if sent_funds.amount < amount {
        return Err(ContractError::InsufficientFunds {
            required: amount,
            provided: sent_funds.amount,
        });
    }

    // Add to the bonus rewards
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.bonus_rewards += amount;
        Ok(config)
    })?;

    // Return success response
    Ok(Response::new().add_attributes(vec![
        ("action", "add_bonus_reward"),
        ("sender", info.sender.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

pub fn execute_update_draw_state(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_state: DrawState,
) -> Result<Response, ContractError> {
    // Verify the caller is the owner
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Update the state
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.draw_state = new_state.clone();
        Ok(config)
    })?;

    // Return success response
    Ok(Response::new().add_attributes(vec![
        ("action", "update_draw_state"),
        ("new_state", &format!("{:?}", new_state)),
    ]))
}

pub fn execute_send_funds(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Verify the caller is the owner
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Create the send message
    let recipient_str = recipient.clone();
    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient,
        amount: vec![CosmosCoin {
            denom: config.core_denom,
            amount,
        }],
    });

    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("action", "send_funds")
        .add_attribute("recipient", &recipient_str)
        .add_attribute("amount", &amount.to_string()))
}

pub fn execute_set_undelegation_timestamp(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    timestamp: u64,
) -> Result<Response, ContractError> {
    // Verify the caller is the owner
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Verify the draw is in the correct state
    if config.draw_state != DrawState::WinnerSelectedUndelegationInProcess {
        return Err(ContractError::InvalidDrawState {
            expected: DrawState::WinnerSelectedUndelegationInProcess,
            actual: config.draw_state,
        });
    }

    // Update the timestamp
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.undelegation_done_timestamp = Some(timestamp);
        Ok(config)
    })?;

    // Return success response
    Ok(Response::new().add_attributes(vec![
        ("action", "set_undelegation_timestamp"),
        ("timestamp", &timestamp.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { account } => to_json_binary(&query_ticket_balance(deps, account)?),
        QueryMsg::GetParticipants {} => to_json_binary(&query_participants(deps)?),
        QueryMsg::GetWinner {} => to_json_binary(&query_winner(deps)?),
        QueryMsg::GetCurrentState {} => to_json_binary(&query_current_state(deps)?),
        QueryMsg::GetNumberOfTicketsSold {} => to_json_binary(&query_number_of_tickets_sold(deps)?),
        QueryMsg::GetBonusRewards {} => to_json_binary(&query_bonus_rewards(deps)?),
        QueryMsg::GetAccumulatedRewards {} => {
            to_json_binary(&query_accumulated_rewards(deps, &_env)?)
        }
        QueryMsg::GetAccumulatedRewardsAtUndelegation {} => {
            to_json_binary(&query_accumulated_rewards_at_undelegation(deps)?)
        }
        QueryMsg::GetDraftTvl {} => to_json_binary(&query_draft_tvl(deps)?),
        QueryMsg::GetTicketHolders {} => to_json_binary(&query_ticket_holders(deps)?),
        QueryMsg::GetUserNumberOfTickets { address } => {
            to_json_binary(&query_user_number_of_tickets(deps, address)?)
        }
        QueryMsg::GetUserWinChance { address } => {
            to_json_binary(&query_user_win_chance(deps, address)?)
        }
        QueryMsg::GetTotalTicketsBurned {} => to_json_binary(&query_total_tickets_burned(deps)?),
        QueryMsg::GetClaims { address } => to_json_binary(&query_claims(deps, address)?),
        QueryMsg::GetDelegatedAmount {} => to_json_binary(&query_delegated_amount(deps, &_env)?),
        QueryMsg::GetContractConfig {} => to_json_binary(&query_contract_config(deps)?),
    }
}

// Query functions

fn query_contract_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_accumulated_rewards_at_undelegation(
    deps: Deps,
) -> StdResult<AccumulatedRewardsAtUndelegationResponse> {
    let accumulated_rewards = ACCUMALTED_REWARDS_AT_UNDELEGATION.load(deps.storage)?;
    Ok(AccumulatedRewardsAtUndelegationResponse {
        accumulated_rewards,
    })
}

fn query_accumulated_rewards(deps: Deps, env: &Env) -> StdResult<AccumulatedRewardsResponse> {
    let config = CONFIG.load(deps.storage)?;
    let coreum_labs_validator = config.validator_address;
    let rewards = deps
        .querier
        .query_delegation_rewards(env.contract.address.to_string(), coreum_labs_validator)?;
    let mut accumulated_rewards = Uint128::zero();
    for dec_coin in rewards {
        if dec_coin.denom == config.core_denom {
            accumulated_rewards += dec_coin
                .amount
                .to_uint_floor()
                .try_into()
                .unwrap_or(Uint128::zero());
        }
    }
    Ok(AccumulatedRewardsResponse {
        accumulated_rewards,
    })
}

fn query_ticket_balance(deps: Deps, account: String) -> StdResult<QueryBalanceResponse> {
    let denom = TICKET_DENOM.load(deps.storage)?;
    let request = QueryBalanceRequest { account, denom };
    request.query(&deps.querier)
}

fn query_participants(deps: Deps) -> StdResult<ParticipantsResponse> {
    let mut participants = vec![];
    let total_tickets_sold = TOTAL_TICKETS_SOLD.load(deps.storage)?;

    // Iterate through all ticket holders
    let all_ticket_holders: Vec<(Addr, Uint128)> = TICKET_HOLDERS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    for (addr, tickets) in all_ticket_holders {
        if !tickets.is_zero() {
            participants.push(ParticipantInfo {
                address: addr.to_string(),
                tickets,
                win_chance: calculate_win_chance(tickets, total_tickets_sold),
            });
        }
    }

    let total_participants = participants.len() as u64;
    Ok(ParticipantsResponse {
        participants,
        total_participants,
    })
}

fn query_winner(deps: Deps) -> StdResult<WinnerResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(WinnerResponse {
        winner: config.winner.map(|addr| addr.to_string()),
        rewards: config.accumulated_rewards + config.bonus_rewards,
    })
}

fn query_current_state(deps: Deps) -> StdResult<CurrentStateResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(CurrentStateResponse {
        state: config.draw_state,
        undelegation_done_timestamp: config.undelegation_done_timestamp,
    })
}

fn query_number_of_tickets_sold(deps: Deps) -> StdResult<TicketsSoldResponse> {
    let config = CONFIG.load(deps.storage)?;
    let tickets_sold = TOTAL_TICKETS_SOLD.load(deps.storage)?;

    Ok(TicketsSoldResponse {
        total_tickets: config.total_tickets,
        tickets_sold,
        tickets_remaining: config.total_tickets - tickets_sold,
    })
}

fn query_bonus_rewards(deps: Deps) -> StdResult<BonusRewardsResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(BonusRewardsResponse {
        bonus_rewards: config.bonus_rewards,
    })
}

fn query_draft_tvl(deps: Deps) -> StdResult<DraftTvlResponse> {
    let config = CONFIG.load(deps.storage)?;
    let tvl = get_draft_tvl(deps.storage)?;

    Ok(DraftTvlResponse {
        tvl,
        denom: config.core_denom,
    })
}

fn query_ticket_holders(deps: Deps) -> StdResult<TicketHoldersResponse> {
    let mut holders = vec![];
    let total_tickets_sold = TOTAL_TICKETS_SOLD.load(deps.storage)?;

    // Iterate through all ticket holders
    let all_ticket_holders: Vec<(Addr, Uint128)> = TICKET_HOLDERS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    for (addr, tickets) in all_ticket_holders {
        if !tickets.is_zero() {
            holders.push(ParticipantInfo {
                address: addr.to_string(),
                tickets,
                win_chance: calculate_win_chance(tickets, total_tickets_sold),
            });
        }
    }

    Ok(TicketHoldersResponse {
        holders: holders.clone(),
        total_holders: holders.len() as u64,
    })
}

fn query_user_number_of_tickets(deps: Deps, address: String) -> StdResult<UserTicketsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let tickets = TICKET_HOLDERS
        .may_load(deps.storage, &addr)?
        .unwrap_or(Uint128::zero());

    Ok(UserTicketsResponse { address, tickets })
}

fn query_user_win_chance(deps: Deps, address: String) -> StdResult<UserWinChanceResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let tickets = TICKET_HOLDERS
        .may_load(deps.storage, &addr)?
        .unwrap_or(Uint128::zero());

    let total_tickets_sold = TOTAL_TICKETS_SOLD.load(deps.storage)?;

    Ok(UserWinChanceResponse {
        address,
        tickets,
        win_chance: calculate_win_chance(tickets, total_tickets_sold),
    })
}

fn query_total_tickets_burned(deps: Deps) -> StdResult<TotalBurnedResponse> {
    let total_burned = TOTAL_TICKETS_BURNED.load(deps.storage)?;

    Ok(TotalBurnedResponse { total_burned })
}

fn query_claims(deps: Deps, address: Option<String>) -> StdResult<ClaimsResponse> {
    let mut claims = vec![];
    let mut total_claimed = Uint128::zero();

    match address {
        Some(addr) => {
            // Query specific address claim
            let addr = deps.api.addr_validate(&addr)?;
            let claim_amount = CLAIMS
                .may_load(deps.storage, &addr)?
                .unwrap_or(Uint128::zero());

            if !claim_amount.is_zero() {
                claims.push(ClaimInfo {
                    address: addr.to_string(),
                    amount: claim_amount,
                });
                total_claimed = claim_amount;
            }
        }
        None => {
            // Query all claims
            let all_claims: Vec<(Addr, Uint128)> = CLAIMS
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<_>>>()?;

            for (addr, amount) in all_claims {
                claims.push(ClaimInfo {
                    address: addr.to_string(),
                    amount,
                });
                total_claimed += amount;
            }
        }
    }

    Ok(ClaimsResponse {
        claims,
        total_claimed,
    })
}

fn query_delegated_amount(deps: Deps, env: &Env) -> StdResult<DelegatedAmountResponse> {
    let config = CONFIG.load(deps.storage)?;
    let delegation = deps.querier.query_delegation(
        env.contract.address.to_string(),
        config.validator_address.clone(),
    )?;

    let amount = delegation
        .map(|d| Coin {
            denom: d.amount.denom.clone(),
            amount: d.amount.amount.to_string(),
        })
        .unwrap_or(Coin {
            denom: config.core_denom,
            amount: "0".to_string(),
        });

    Ok(DelegatedAmountResponse { amount })
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
//     let version = cw2::get_contract_version(deps.storage)?;

//     // Ensure we're migrating from the same contract
//     if version.contract != CONTRACT_NAME {
//         return Err(ContractError::InvalidMigration {
//             current_name: version.contract,
//             current_version: version.version,
//         });
//     }

//     // Update contract version
//     set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

//     // Update validator address if provided
//     if let Some(new_validator) = msg.new_validator_address {
//         // Validate the new validator address
//         deps.api.addr_validate(&new_validator)?;

//         CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
//             config.validator_address = new_validator.clone();
//             Ok(config)
//         })?;
//     }

//     Ok(Response::new()
//         .add_attribute("action", "migrate")
//         .add_attribute("from_version", version.version)
//         .add_attribute("to_version", CONTRACT_VERSION))
// }
