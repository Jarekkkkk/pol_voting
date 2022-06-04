mod registrar;
pub use registrar::Registrar;

mod exchange_rate_entry;
pub use exchange_rate_entry::ExchangeRateEntry;

mod voter;
pub use voter::Voter;

mod deposit_entry;
pub use deposit_entry::DepositEntry;

mod lockup;
pub use lockup::{Lockup, LockupKind};
//logic first --> reprc(C), using Zeroable trait and sub-trait "POD" --> adding macro to write DRY
