//no need of using "solana-test-validator"
mod action;

use solana_program_test::*;

use solana_sdk::{account::Account, signature::Keypair, signer::Signer};

//instruction
use program::entrypoint::process_instruction;

#[tokio::test]
async fn test() {
    use program::instruction;

    let mut pt = ProgramTest::new("program", program::id(), processor!(process_instruction));
    pt.set_compute_max_units(5_000);

    // === progrma_config ===
    // add voter to the context
    // let voter = Keypair::new();
    // let _voter_pubkey = voter.pubkey();
    // pt.add_account(
    //     voter.pubkey(),
    //     Account {
    //         lamports: 1_000000000000000,
    //         ..Account::default()
    //     },
    // );

    let (mut banks_client, payer, _recent_blockhash) = pt.start().await;

    //prerequisites
    //: 1.create Mint and Vault
    //: 2.find PDA
    //: respective ix
    //const
    let _realm = Keypair::new();
    let _rent = banks_client.get_rent().await.unwrap();
    let _owner = payer;
}
