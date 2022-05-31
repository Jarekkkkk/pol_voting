mod registrar;
pub use registrar::Registrar;

mod exchange_rate_entry;
pub use exchange_rate_entry::ExchangeRateEntry;

//logic first --> reprc(C), using Zeroable trait and sub-trait "POD" --> adding macro to write DRY
