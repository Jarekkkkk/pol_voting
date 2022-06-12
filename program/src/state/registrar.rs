use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

//Account
use crate::{error::GovError, state::ExchangeRateEntry};

//exchange rate for an asset that can mint the voting rights
#[derive(Debug, BorshDeserialize, BorshSchema, BorshSerialize, Default, Copy, Clone, PartialEq)]
pub struct Registrar {
    pub authority: Pubkey,            //set the role as authority
    pub realm: Pubkey,                // from random pubkey
    pub realm_community_mint: Pubkey, // our POL mint
    pub bump: u8,                     //helpful for invoke_signed

    pub rates: [ExchangeRateEntry; 2], // The length should be adjusted for one's use case.

    pub rate_decimals: u8, // The decimals to use when converting deposits into a common currency.
}

impl Registrar {
    //convert the given amount into community-based currency
    //update both
    //  1: exchagne rate conversion
    //  2: decimals conversion
    pub fn convert(&self, er: &ExchangeRateEntry, amount: u64) -> Result<u64, ProgramError> {
        if !(self.rate_decimals >= er.decimals) {
            return Err(GovError::InvalidDecimals.into());
        }

        let decimals_diff = self.rate_decimals.checked_sub(er.decimals).unwrap();
        let convert = amount
            .checked_mul(er.rate)
            .unwrap()
            .checked_mul(10_u64.pow(decimals_diff.into()))
            .unwrap();

        Ok(convert)
    }
}
