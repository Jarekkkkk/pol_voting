use crate::utils::account_info_util::Acc;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Exchange rate for an asset that can be used to mint voting rights
#[derive(BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone, Copy, Debug, PartialEq)]
pub struct ExchangeRateEntry {
    pub mint: Pubkey, //mint for this entry
    pub rate: u64,    // Exchange rate into the common currency.
    pub decimals: u8, // Mint decimals.
}

impl Acc for ExchangeRateEntry {
    fn get_max_size(&self) -> Option<usize> {
        Some(32 + 8 + 1)
    }
}

impl ExchangeRateEntry {
    pub fn get_voting_mint_seeds<'a>(
        registrar: &'a Pubkey,
        deposit_mint: &'a Pubkey,
    ) -> [&'a [u8]; 2] {
        [registrar.as_ref(), deposit_mint.as_ref()]
    }
}
