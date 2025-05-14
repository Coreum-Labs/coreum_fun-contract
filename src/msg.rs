use crate::state::DrawState;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Denom of the TICKET token (will be created on contract init)
    pub ticket_token: String,
    /// Coreum Labs validator address for staking
    pub validator_address: String,
    /// Total number of tickets available for the draft
    pub total_tickets: Uint128,
    /// Price per ticket in CORE
    pub ticket_price: Uint128,
    /// Maximum number of tickets per user
    pub max_tickets_per_user: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Buy one or multiple tickets
    BuyTicket { number_of_tickets: Uint128 },

    /// Select the winner and send funds (admin only)
    SelectWinnerAndSendFunds { winner_address: String },

    /// Burn tickets to get the principal back
    BurnTickets { number_of_tickets: Uint128 },

    /// Add extra rewards to the pool
    AddBonusRewardToThePool { amount: Uint128 },

    /// Manually update the draw state (admin only)
    UpdateDrawState { new_state: DrawState },

    /// Send funds to a recipient
    SendFunds { recipient: String, amount: Uint128 },

    /// Manually set the undelegation timestamp (admin only)
    SetUndelegationTimestamp { timestamp: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the balance of a specific address
    #[returns(BalanceResponse)]
    Balance { account: String },

    /// Get all participants in the lottery
    #[returns(ParticipantsResponse)]
    GetParticipants {},

    /// Get the current winner if selected
    #[returns(WinnerResponse)]
    GetWinner {},

    /// Get the current state of the draw
    #[returns(CurrentStateResponse)]
    GetCurrentState {},

    /// Get the total number of tickets sold
    #[returns(TicketsSoldResponse)]
    GetNumberOfTicketsSold {},

    /// Get the bonus rewards added to the pool
    #[returns(BonusRewardsResponse)]
    GetBonusRewards {},

    /// Get the total accumulated rewards
    #[returns(AccumulatedRewardsResponse)]
    GetAccumulatedRewards {},

    /// Get the total value locked in the draft
    #[returns(DraftTvlResponse)]
    GetDraftTvl {},

    /// Get all ticket holders
    #[returns(TicketHoldersResponse)]
    GetTicketHolders {},

    /// Get number of tickets owned by a user
    #[returns(UserTicketsResponse)]
    GetUserNumberOfTickets { address: String },

    /// Get a user's chance of winning
    #[returns(UserWinChanceResponse)]
    GetUserWinChance { address: String },

    /// Get total burned tickets
    #[returns(TotalBurnedResponse)]
    GetTotalTicketsBurned {},

    /// Get total claims made by users
    #[returns(ClaimsResponse)]
    GetClaims { address: Option<String> },
}

/// Migration message for contract upgrades
#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct AccumulatedRewardsResponse {
    pub accumulated_rewards: Uint128,
}

pub struct MigrateMsg {
    pub new_validator_address: Option<String>,
}

/// Response structures for queries

#[cw_serde]
pub struct ParticipantsResponse {
    pub participants: Vec<ParticipantInfo>,
    pub total_participants: u64,
}

#[cw_serde]
pub struct ParticipantInfo {
    pub address: String,
    pub tickets: Uint128,
    pub win_chance: String, // Formatted as percentage
}

#[cw_serde]
pub struct WinnerResponse {
    pub winner: Option<String>,
    pub rewards: Uint128,
}

#[cw_serde]
pub struct CurrentStateResponse {
    pub state: DrawState,
    pub undelegation_done_timestamp: Option<u64>,
}

#[cw_serde]
pub struct TicketsSoldResponse {
    pub total_tickets: Uint128,
    pub tickets_sold: Uint128,
    pub tickets_remaining: Uint128,
}

#[cw_serde]
pub struct BonusRewardsResponse {
    pub bonus_rewards: Uint128,
}

#[cw_serde]
pub struct DraftTvlResponse {
    pub tvl: Uint128,
    pub denom: String,
}

#[cw_serde]
pub struct TicketHoldersResponse {
    pub holders: Vec<ParticipantInfo>,
    pub total_holders: u64,
}

#[cw_serde]
pub struct UserTicketsResponse {
    pub address: String,
    pub tickets: Uint128,
}

#[cw_serde]
pub struct UserWinChanceResponse {
    pub address: String,
    pub tickets: Uint128,
    pub win_chance: String, // Formatted as percentage
}

#[cw_serde]
pub struct TotalBurnedResponse {
    pub total_burned: Uint128,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<ClaimInfo>,
    pub total_claimed: Uint128,
}

#[cw_serde]
pub struct ClaimInfo {
    pub address: String,
    pub amount: Uint128,
}

// For Coreum asset FT module token holders query
#[cw_serde]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub key: Option<String>,
    pub count_total: Option<bool>,
    pub reverse: Option<bool>,
}

#[cw_serde]
pub struct TokenHoldersResponse {
    pub holders: Vec<TokenHolder>,
    pub pagination: Option<PaginationResponse>,
}

#[cw_serde]
pub struct TokenHolder {
    pub address: String,
    pub balance: Uint128,
}

#[cw_serde]
pub struct PaginationResponse {
    pub next_key: Option<String>,
    pub total: Option<u64>,
}
