//no need of using "solana-test-validator"
mod action;

use solana_program_test::*;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use program::entrypoint::process_instruction;

#[tokio::test]
async fn test() {
    //use program::instruction;

    let mut pt = ProgramTest::new("program", program::id(), processor!(process_instruction));
    pt.set_compute_max_units(5_000); //per tx

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

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    //prerequisites
    let mint_a = Keypair::new();
    let vault_a = Keypair::new();
    let mint_b = Keypair::new();
    let vault_b = Keypair::new();

    let realm = Keypair::new();
    let _rent = banks_client.get_rent().await.unwrap();

    // ------ Mint & Vault ------
    action::create_token_and_mint(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &program::id(),
        &mint_a,
        8,
        &vault_a,
    )
    .await
    .unwrap();
    action::create_token_and_mint(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &program::id(),
        &mint_b,
        0,
        &vault_b,
    )
    .await
    .unwrap();
    // ------ ------

    // ------ PDA ------
    let (registrar_pda, registrar_bump) =
        Pubkey::find_program_address(&[&realm.pubkey().as_ref()], &program::id());

    let (_voting_mint_pda, _voting_mint_bump) = Pubkey::find_program_address(
        &[
            registrar_pda.as_ref(),
            mint_a.pubkey().as_ref(),
            spl_token::id().as_ref(),
        ],
        &spl_associated_token_account::id(),
    );
    // ------ ------

    // ------ create_registrar ------
    action::create_registrar(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &realm.pubkey(),
        &payer.pubkey(),
        &mint_a.pubkey(),
        6,
    )
    .await
    .unwrap();
}
