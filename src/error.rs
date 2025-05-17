use crate::state::DrawState;
use cosmwasm_std::{StdError, Uint128};
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid ticket amount")]
    InvalidTicketAmount {},

    #[error("Invalid ticket price")]
    InvalidTicketPrice {},

    #[error("Ticket sales are closed")]
    TicketSalesClosed {},

    #[error("Not enough tickets left (requested: {requested:?}, available: {available:?})")]
    NotEnoughTicketsLeft {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Max tickets per user reached (requested: {requested:?}, available: {available:?})")]
    MaxTicketsPerUserReached {
        requested: Uint128,
        available: Uint128,
    },

    #[error("No funds sent")]
    NoFunds {},

    #[error("Insufficient funds (required: {required:?}, provided: {provided:?})")]
    InsufficientFunds {
        required: Uint128,
        provided: Uint128,
    },

    #[error("No tickets found for address")]
    NoTicketsForAddress {},

    #[error("Invalid draw state (expected: {expected:?}, actual: {actual:?})")]
    InvalidDrawState {
        expected: DrawState,
        actual: DrawState,
    },

    #[error("Not enough tickets (requested: {requested:?}, available: {available:?})")]
    NotEnoughTickets {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Undelegation period not completed (current timestamp: {current_timestamp:?}, undelegation timestamp: {undelegation_timestamp:?})")]
    UndelegationPeriodNotCompleted {
        current_timestamp: u64,
        undelegation_timestamp: u64,
    },

    #[error("No undelegation in progress")]
    NoUndelegationInProgress {},

    #[error("Invalid state transition (from: {from:?}, to: {to:?})")]
    InvalidStateTransition { from: String, to: String },

    #[error("Cannot close ticket sales until all tickets are sold")]
    CannotCloseTicketSales {},

    #[error("Use select_winner function to set winner and start undelegation")]
    UseSelectWinnerFunction {},

    #[error("Not all tickets have been burned")]
    NotAllTicketsBurned {},

    #[error("Invalid migration (current contract: {current_name:?}, current version: {current_version:?})")]
    InvalidMigration {
        current_name: String,
        current_version: String,
    },

    #[error("Invalid token parameters")]
    InvalidTokenParameters {},

    #[error("Token already issued")]
    TokenAlreadyIssued {},

    #[error("Failed to query token metadata")]
    TokenQueryFailed {},

    #[error("Failed to delegate tokens")]
    DelegationFailed {},

    #[error("Failed to undelegate tokens")]
    UndelegationFailed {},

    #[error("Failed to distribute rewards")]
    RewardsDistributionFailed {},

    #[error("Invalid address: {address:?}")]
    InvalidAddress { address: String },

    #[error("Contract is paused")]
    ContractPaused {},

    #[error("Overflow")]
    Overflow {},

    #[error("Failed to calculate accumulated rewards")]
    RewardsCalculationFailed {},

    #[error("Invalid query")]
    InvalidQuery {},

    #[error("No winner has been selected yet")]
    NoWinnerSelected {},

    #[error("No rewards to send")]
    NoRewardsToSend {},
}
