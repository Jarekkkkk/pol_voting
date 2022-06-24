use std::cell::{Ref, RefMut};

use crate::{
    error::GovError,
    state::{Lockup, Registrar, Voter},
};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

/// Bookkeeping for a single deposit for a given mint and lockup schedule.
#[derive(BorshDeserialize, BorshSerialize, PartialEq, BorshSchema, Default, Copy, Clone, Debug)]
pub struct DepositEntry {
    // True if the deposit entry is being used.
    pub is_used: bool,

    // Points to the ExchangeRate this deposit uses.
    pub rate_idx: u8,

    // Amount in the native currency deposited.
    pub amount_deposited: u64,

    // Amount withdrawn from the deposit in the native currency.
    pub amount_withdrawn: u64,

    // Amount in the native currency deposited, scaled by the exchange rate.
    pub amount_scaled: u64,

    // Locked state.
    pub lockup: Lockup,
}

//could be optimized by RefMut (?)
impl DepositEntry {
    pub fn update_deposit(
        voter: &mut Voter, //to access the struct state
        registrar: &Registrar,
        update_idx: u8,
        amount: u64,
        voter_info: &AccountInfo,
        deposit_mint: &AccountInfo,
    ) -> ProgramResult {
        let amount_scaled = {
            let er_idx = registrar
                .rates
                .iter()
                .position(|i| i.mint == *deposit_mint.key)
                .ok_or(GovError::ExchangeRateEntryNotFound)?;
            let er = registrar.rates[er_idx];
            registrar.convert(&er, amount)?
        };
        //verify
        if !(voter.deposits.len() > update_idx as usize) {
            return Err(GovError::InvalidDepositId.into());
        }

        //logic
        let mut d_er = &mut voter.deposits[update_idx as usize];
        d_er.amount_deposited += amount;
        d_er.amount_scaled += amount_scaled;

        msg!("d_er{:?}", d_er);

        //serialize
        voter.serialize(&mut *voter_info.try_borrow_mut_data()?)?;

        Ok(())
    }
}
