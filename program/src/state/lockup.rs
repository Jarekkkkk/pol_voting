use borsh::{BorshDeserialize, BorshSerialize};

/// Seconds in one day.
pub const _SECS_PER_DAY: i64 = 86_400;

/// Maximum number of days one can lock for.
pub const _MAX_DAYS_LOCKED: u64 = 2555;

#[derive(BorshDeserialize, BorshSerialize, Default, Clone, Copy, Debug)]
pub struct Lockup {
    pub kind: LockupKind,
    // Start of the lockup.
    pub start_ts: i64,
    // End of the lockup.
    pub end_ts: i64,
    // Empty bytes for future upgrades.
    pub padding: [u8; 16],
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug)]
pub enum LockupKind {
    Daily,
    Cliff,
}

impl Default for LockupKind {
    fn default() -> Self {
        LockupKind::Cliff
    }
}
