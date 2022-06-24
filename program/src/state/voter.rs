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
}
