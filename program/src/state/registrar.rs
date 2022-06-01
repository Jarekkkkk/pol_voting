use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

//Account
use crate::state::ExchangeRateEntry;

//exchange rate for an asset that can mint the voting rights
#[derive(Debug, BorshDeserialize, BorshSerialize, Default)]
pub struct Registrar {
    pub authority: Pubkey,            //set the role as authority
    pub realm: Pubkey,                // from random pubkey
    pub realm_community_mint: Pubkey, // one of token mint we created, lieky becoming our governant token
    pub bump: u8,                     //helpful for invoke_signed
    // The length should be adjusted for one's use case.
    pub rates: [ExchangeRateEntry; 2],
    // The decimals to use when converting deposits into a common currency.
    pub rate_decimals: u8,
}

impl Registrar {
    pub fn serialized_size() -> usize {
        Self::default()
            .try_to_vec()
            .expect("seriazlied length: Registrar")
            .len()
    }
}
