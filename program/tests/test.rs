//no need of using "solana-test-validator"
mod action;

use solana_program_test::*;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use program::entrypoint::process_instruction;

#[tokio::test]
async fn test() {
    //use program::instruction;

    let pt = ProgramTest::new("program", program::id(), processor!(process_instruction));
    //pt.set_compute_max_units(5_000); //per tx

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
    let seeds: &[&[_]] = &[&realm.pubkey().to_bytes().clone()];
    let (registrar_pda, registrar_bump) = Pubkey::find_program_address(seeds, &program::id());

    //voting_mint_a
    let seeds: &[&[_]] = &[
        &registrar_pda.to_bytes().clone(),
        &mint_a.pubkey().to_bytes().clone(),
    ];
    let (voting_mint_a_pda, voting_mint_a_bump) =
        Pubkey::find_program_address(seeds, &program::id());

    //voting_mint_b
    let seeds: &[&[_]] = &[
        &registrar_pda.to_bytes().clone(),
        &mint_b.pubkey().to_bytes().clone(),
    ];
    let (voting_mint_b_pda, voting_mint_b_bump) =
        Pubkey::find_program_address(seeds, &program::id());

    //exchange_vault_a
    //ATA simply is PDA derived from [owner,mint,token_program]

    let exchange_vault_a_pda = spl_associated_token_account::get_associated_token_address(
        &registrar_pda,
        &mint_a.pubkey(),
    );
    let exchange_vault_b_pda = spl_associated_token_account::get_associated_token_address(
        &registrar_pda,
        &mint_b.pubkey(),
    );
    // let seeds: &[&[_]] = &[
    //     &registrar_pda.to_bytes().clone(),
    //     &spl_token::id().to_bytes().clone(),
    //     &mint_a.pubkey().to_bytes().clone(),
    // ];
    // let (exchange_vault_a_pda, _exchange_vault_a_bump) =
    //     Pubkey::find_program_address(seeds, &spl_associated_token_account::id());

    // ------ create_registrar ------
    action::create_registrar(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &realm.pubkey(),
        &payer.pubkey(),
        &mint_a.pubkey(),
        registrar_pda,
        registrar_bump,
        6,
    )
    .await
    .unwrap();

    // ------ create_exchange_rate A ------
    // no need to assign `exchange_vault_a_bump`,
    // since ATA program do us the favor for
    // creating PDA themselves
    let er_a = program::state::ExchangeRateEntry {
        mint: mint_a.pubkey(),
        rate: 10,
        decimals: 6,
    };
    action::create_exchange_rate(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &payer.pubkey(),
        &registrar_pda,
        &mint_a.pubkey(),
        &exchange_vault_a_pda,
        &voting_mint_a_pda,
        voting_mint_a_bump,
        0,
        er_a,
    )
    .await
    .unwrap();

    // ------ create_exchange_rate B ------
    let er_b = program::state::ExchangeRateEntry {
        mint: mint_b.pubkey(),
        rate: 1_000_000,
        decimals: 0,
    };
    action::create_exchange_rate(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &payer.pubkey(),
        &registrar_pda,
        &mint_b.pubkey(),
        &exchange_vault_b_pda,
        &voting_mint_b_pda,
        voting_mint_b_bump,
        1,
        er_b,
    )
    .await
    .unwrap();
}
