use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum DrawState {
    TicketSalesOpen,                        // Initial state, tickets can be bought
    TicketsSoldOutAccumulationInProgress,   // All tickets sold, rewards accumulating
    WinnerSelectedUndelegationInProcess,    // Winner selected, waiting for undelegation
    UndelegationCompletedTokensCanBeBurned, // Undelegation completed, tickets can be burned
    DrawFinished,                           // All tickets burned, draw cycle complete
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,                              // Contract owner
    pub ticket_token: String,                     // Denom of the TICKET token
    pub core_denom: String,                       // Denom of CORE token (ucore)
    pub validator_address: String,                // Coreum Labs validator address
    pub total_tickets: Uint128,                   // Total number of tickets available
    pub max_tickets_per_user: Uint128,            // Maximum number of tickets per user
    pub ticket_price: Uint128,                    // Price per ticket in CORE
    pub draw_state: DrawState,                    // Current state of the draw
    pub winner: Option<Addr>,                     // Winner address (if selected)
    pub undelegation_done_timestamp: Option<u64>, // Timestamp at which undelegation will complete
    pub accumulated_rewards: Uint128,             // Total rewards accumulated
    pub bonus_rewards: Uint128,                   // Additional bonus rewards
}

// Key storage items
pub const CONFIG: Item<Config> = Item::new("config");
pub const TICKET_HOLDERS: Map<&Addr, Uint128> = Map::new("ticket_holders"); // Address -> Number of tickets
pub const TOTAL_TICKETS_SOLD: Item<Uint128> = Item::new("total_tickets_sold");
pub const TOTAL_TICKETS_BURNED: Item<Uint128> = Item::new("total_tickets_burned");
pub const CLAIMS: Map<&Addr, Uint128> = Map::new("claims"); // Address -> Amount claimed

// Initialize the storage
pub fn initialize_storage(storage: &mut dyn Storage) -> StdResult<()> {
    TOTAL_TICKETS_SOLD.save(storage, &Uint128::zero())?;
    TOTAL_TICKETS_BURNED.save(storage, &Uint128::zero())?;
    Ok(())
}

// Helper functions to work with state

pub fn increment_tickets_sold(storage: &mut dyn Storage, amount: Uint128) -> StdResult<Uint128> {
    TOTAL_TICKETS_SOLD.update(storage, |current| -> StdResult<_> { Ok(current + amount) })
}

pub fn increment_tickets_burned(storage: &mut dyn Storage, amount: Uint128) -> StdResult<Uint128> {
    TOTAL_TICKETS_BURNED.update(storage, |current| -> StdResult<_> { Ok(current + amount) })
}

pub fn update_ticket_holder(
    storage: &mut dyn Storage,
    addr: &Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    TICKET_HOLDERS.update(storage, addr, |current| -> StdResult<_> {
        match current {
            Some(value) => Ok(value + amount),
            None => Ok(amount),
        }
    })
}

pub fn decrease_ticket_holder(
    storage: &mut dyn Storage,
    addr: &Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    TICKET_HOLDERS.update(storage, addr, |current| -> StdResult<_> {
        match current {
            Some(value) => {
                if value < amount {
                    return Err(cosmwasm_std::StdError::generic_err("Not enough tickets"));
                }
                if value == amount {
                    return Ok(Uint128::zero());
                }
                Ok(value - amount)
            }
            None => Err(cosmwasm_std::StdError::generic_err("No tickets found")),
        }
    })
}

pub fn update_claim(storage: &mut dyn Storage, addr: &Addr, amount: Uint128) -> StdResult<Uint128> {
    CLAIMS.update(storage, addr, |current| -> StdResult<_> {
        match current {
            Some(value) => Ok(value + amount),
            None => Ok(amount),
        }
    })
}

pub fn get_draft_tvl(storage: &dyn Storage) -> StdResult<Uint128> {
    let config = CONFIG.load(storage)?;
    let total_sold = TOTAL_TICKETS_SOLD.load(storage)?;
    Ok(total_sold * config.ticket_price)
}

pub fn should_close_ticket_sales(storage: &dyn Storage) -> StdResult<bool> {
    let config = CONFIG.load(storage)?;
    let total_sold = TOTAL_TICKETS_SOLD.load(storage)?;
    Ok(total_sold == config.total_tickets)
}

pub fn all_tickets_burned(storage: &dyn Storage) -> StdResult<bool> {
    let total_sold = TOTAL_TICKETS_SOLD.load(storage)?;
    let total_burned = TOTAL_TICKETS_BURNED.load(storage)?;
    Ok(total_sold == total_burned)
}

pub fn calculate_win_chance(user_tickets: Uint128, total_tickets_sold: Uint128) -> String {
    if total_tickets_sold.is_zero() || user_tickets.is_zero() {
        return "0.00%".to_string();
    }

    let win_chance = user_tickets.u128() as f64 / total_tickets_sold.u128() as f64 * 100.0;
    format!("{:.2}%", win_chance)
}
