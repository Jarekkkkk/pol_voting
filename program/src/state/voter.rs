use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{error::GovError, state::DepositEntry, utils::account_info_util::Acc};

use spl_governance_addin_api::voter_weight::VoterWeightRecord;

#[derive(BorshDeserialize, PartialEq, BorshSerialize, BorshSchema, Default, Clone, Debug)]
pub struct Voter {
    pub authority: Pubkey,
    pub registrar: Pubkey,
    pub voter_bump: u8,               // for state::Voter
    pub voter_weight_record_bump: u8, //for state::VoterWeightRecord
    pub deposits: [DepositEntry; 10], //bookkeeping records of individual assets
}

impl Acc for Voter {}
impl Acc for VoterWeightRecord {}

impl Voter {
    pub fn assert_voter(&self, authority: &Pubkey, registrar: &Pubkey) -> Result<(), ProgramError> {
        if self.registrar != *registrar {
            return Err(GovError::RegistrarMismatch.into());
        };
        if self.authority != *authority {
            return Err(GovError::AuthorityMismatch.into());
        };

        Ok(())
    }

    pub fn get_voter_seeds<'a>(registrar: &'a Pubkey, authority: &'a Pubkey) -> [&'a [u8]; 2] {
        [registrar.as_ref(), authority.as_ref()]
    }
    pub fn get_voter_weight_seeds<'a>(
        registrar: &'a Pubkey,
        authority: &'a Pubkey,
    ) -> [&'a [u8]; 3] {
        const VOTER_WEIGHT_RECORD: [u8; 19] = *b"voter-weight-record";
        [
            VOTER_WEIGHT_RECORD.as_ref(),
            registrar.as_ref(),
            authority.as_ref(),
        ]
    }
}
