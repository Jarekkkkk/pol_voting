use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

/// Seconds in one day.
pub const SECS_PER_DAY: i64 = 86_400;

/// Maximum number of days one can lock for.
pub const MAX_DAYS_LOCKED: u64 = 2555;

#[derive(BorshDeserialize, BorshSerialize, PartialEq, BorshSchema, Default, Copy, Clone, Debug)]
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
    Daily, //linear rewards
    Cliff, // perioduc rewards
}

impl Default for LockupKind {
    fn default() -> Self {
        LockupKind::Cliff
    }
}
