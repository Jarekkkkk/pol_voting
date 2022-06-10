use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

/// Seconds in one day.
pub const SECS_PER_DAY: i64 = 86_400;

/// Maximum number of days one can lock for.
pub const MAX_DAYS_LOCKED: u64 = 1095;

#[derive(BorshDeserialize, BorshSerialize, PartialEq, BorshSchema, Default, Copy, Clone, Debug)]
///Provide ether (1. Daily or Cliff vesting with maximum locked daysv in 3 years
pub struct Lockup {
    pub kind: LockupKind,
    // Start of the lockup.
    pub start_ts: i64,
    // End of the lockup.
    pub end_ts: i64,
    // Empty bytes for future upgrades.
    pub padding: [u8; 16],
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, BorshSchema, Clone, Copy, Debug)]
pub enum LockupKind {
    /// let n = days_left
    /// let m = max_days = 1095
    ///voting_power = (n / m) * amount
    Daily, //linear unlock
    ///voting_power = (1 / m) * (amount / n) * [(n * [(n + 1)]) / 2],
    Cliff, // unlock all at once under specific situation
}

impl Default for LockupKind {
    fn default() -> Self {
        LockupKind::Cliff
    }
}
