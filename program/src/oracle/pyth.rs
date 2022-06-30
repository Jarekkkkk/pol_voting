use arrayref::array_ref;
use fixed::types::I80F48;
//check File id

pub enum Oracles {
    Pyth,
    StubOracle,
    Unknown,
}

pub struct StubOracle {
    pub magic: u32,
    pub price: I80F48,
    pub last_update: u64,
}
