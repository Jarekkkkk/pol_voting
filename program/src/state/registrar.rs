use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

//Account
use crate::{error::GovError, state::ExchangeRateEntry, utils::account_info_util::Acc};

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

impl Acc for Registrar {}

impl Registrar {
    pub fn get_seeds<'a>(realm: &'a Pubkey) -> [&'a [u8]; 1] {
        [realm.as_ref()]
    }
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

    pub fn check_and_get_mut_registrar(
        account: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<Registrar, ProgramError> {
        //dangerous when deref the RefMut

        //{
        // common verification could be optimized
        let registrar = Self::try_from_slice(&account.try_borrow_mut_data()?)?;

        if !account.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if account.data_is_empty() {
            return Err(ProgramError::UninitializedAccount);
        }
        // }

        if registrar.authority != *authority.key {
            let err = GovError::AuthorityMismatch;

            return Err(err.into());
        }

        Ok(registrar)
    }

    pub fn check_and_get_immut_registrar(
        account: &AccountInfo,
        authority: &AccountInfo,
    ) -> Result<Registrar, ProgramError> {
        //dangerous when deref the RefMut
        let registrar = Self::try_from_slice(&account.try_borrow_data()?)?;

        if account.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data_is_empty() {
            return Err(ProgramError::UninitializedAccount);
        }

        if registrar.authority != *authority.key {
            let err = GovError::AuthorityMismatch;

            return Err(err.into());
        }

        Ok(registrar)
    }
}
