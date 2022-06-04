use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::pubkey::Pubkey;

use crate::state::DepositEntry;

#[derive(BorshDeserialize, BorshSerialize, Default, Clone, Copy, Debug)]
pub struct Voter {
    pub authority: Pubkey,
    pub registrar: Pubkey,
    pub voter_bump: u8,               // for state::Voter
    pub voter_weight_record_bump: u8, //for state::VoterWeightRecord
    pub deposits: [DepositEntry; 32], //bookkeeping records of individual assets
}

// #[cfg(test)]
// mod tests {
//     use super::{DepositEntry, Voter};
//     #[test]
//     fn foo() {
//         let foo = [DepositEntry::default(); 32];
//         println!("foo{:?}", foo);
//         let result = 4;

//         assert_eq!(result, 3);
//     }
// }
