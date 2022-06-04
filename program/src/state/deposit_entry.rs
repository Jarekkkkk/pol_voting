use crate::state::Lockup;

use borsh::{BorshDeserialize, BorshSerialize};

/// Bookkeeping for a single deposit for a given mint and lockup schedule.
#[derive(BorshDeserialize, BorshSerialize, Default, Clone, Copy, Debug)]
pub struct DepositEntry {
    // True if the deposit entry is being used.
    pub is_used: bool,

    // Points to the ExchangeRate this deposit uses.
    pub rate_idx: u8,

    // Amount in the native currency deposited.
    pub amount_deposited: u64,

    // Amount withdrawn from the deposit in the native currency.
    pub amount_withdrawn: u64,

    // Amount in the native currency deposited, scaled by the exchange rate.
    pub amount_scaled: u64,

    // Locked state.
    pub lockup: Lockup,
}
