use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use solana_program::pubkey::Pubkey;

use crate::state::DepositEntry;

#[derive(BorshDeserialize, PartialEq, BorshSerialize, BorshSchema, Default, Clone, Debug)]
pub struct Voter {
    pub authority: Pubkey,
    pub registrar: Pubkey,
    pub voter_bump: u8,               // for state::Voter
    pub voter_weight_record_bump: u8, //for state::VoterWeightRecord
    pub deposits: [DepositEntry; 10], //bookkeeping records of individual assets
}
