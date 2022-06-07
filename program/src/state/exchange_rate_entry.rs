use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Exchange rate for an asset that can be used to mint voting rights
#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone, Copy, Debug)]
pub struct ExchangeRateEntry {
    pub mint: Pubkey, //mint for this entry
    pub rate: u64,    // Exchange rate into the common currency.
    pub decimals: u8, // Mint decimals.
}

impl ExchangeRateEntry {
    pub fn serialized_size() -> usize {
        Self::default()
            .try_to_vec()
            .expect("seriazlied length: ExchangeRateEntry")
            .len()
    }
}
