#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::try_err,
    clippy::must_use_candidate,
    clippy::doc_markdown,
    clippy::module_name_repetitions,
    clippy::semicolon_if_nothing_returned,
    clippy::shadow_unrelated,
    clippy::missing_errors_doc,
    clippy::similar_names
)]

use solana_program::declare_id;

pub mod entrypoint;
mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

// @TODO_QUESTION Is possible to read it from "../keypairs/program-pubkey"?
//
// @TODO_QUESTION Should I use declare_program! instead? Does it work with the `no-entrypoint` feature?
// (Do I really need the `no-entrypoint` feature?)
declare_id!("A8bkizaAC3EePjYJjVSzfsUpKqTGREpyb89eT1FJyrzn");

#[cfg(test)]
mod tests {
    use super::*;

    fn two_ways_serialization() {
        use borsh::BorshSerialize;
        use state::Registrar;

        let reg = Registrar::default();
        let seri = reg.try_to_vec().unwrap();
        //const SIZE: usize = 180;

        let mut buffer_1: Vec<u8> = vec![0; 180];
        let mut buffer_2: Vec<u8> = Vec::new();

        buffer_1.copy_from_slice(&seri); //space require identical
        reg.serialize(&mut buffer_2).expect("seri err");

        assert_eq!(buffer_1, buffer_2);
        assert_eq!(buffer_1.len(), buffer_2.len());
    }
}
