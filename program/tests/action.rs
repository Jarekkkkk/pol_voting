use {
    solana_program_test::BanksClient,
    solana_sdk::{
        hash::Hash,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction,
        transaction::Transaction,
        transport::TransportError,
    },
    spl_token::{
        id, instruction,
        state::{Account, Mint},
    },
};

pub async fn create_registrar() -> Result<(), TransportError> {
    Ok(())
}
