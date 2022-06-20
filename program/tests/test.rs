//no need of using "solana-test-validator"
#![allow(unused_variables, unused_imports, dead_code)]
mod action;
mod spl_token_action;
use solana_program_test::*;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};

use program::{entrypoint::process_instruction, state};

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
    spl_token_action::create_token_and_mint(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &payer.pubkey(), //mint authority & token holder
        &mint_a,
        6,
        &vault_a,
    )
    .await
    .unwrap();
    spl_token_action::create_token_and_mint(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &payer.pubkey(),
        &mint_b,
        0,
        &vault_b,
    )
    .await
    .unwrap();
    // ------ ------

    // ------ PDA ------
    //1. registrar
    let seeds: &[&[_]] = &[&realm.pubkey().to_bytes().clone()];
    let (registrar_pda, registrar_bump) = Pubkey::find_program_address(seeds, &program::id());

    //2. voting_mint_a
    let seeds: &[&[_]] = &[
        &registrar_pda.to_bytes().clone(),
        &mint_a.pubkey().to_bytes().clone(),
    ];
    let (voting_mint_a_pda, voting_mint_a_bump) =
        Pubkey::find_program_address(seeds, &program::id());

    //3. voting_mint_b
    let seeds: &[&[_]] = &[
        &registrar_pda.to_bytes().clone(),
        &mint_b.pubkey().to_bytes().clone(),
    ];
    let (voting_mint_b_pda, voting_mint_b_bump) =
        Pubkey::find_program_address(seeds, &program::id());

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

    //exchange_vault_a
    //ATA simply is PDA derived from [owner,mint,token_program]
    let exchange_vault_a_pda = spl_associated_token_account::get_associated_token_address(
        &registrar_pda,
        &mint_a.pubkey(),
    );

    let er_a = program::state::ExchangeRateEntry {
        mint: mint_a.pubkey(),
        rate: 1,
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
    let exchange_vault_b_pda = spl_associated_token_account::get_associated_token_address(
        &registrar_pda,
        &mint_b.pubkey(),
    );
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

    // ------ create_voter ------

    let (voter_pda, voter_bump) = Pubkey::find_program_address(
        &[&registrar_pda.to_bytes(), &payer.pubkey().to_bytes()],
        &program::id(),
    );
    let seeds: &[&[_]] = &[
        &program::processor::create_voter::VOTER_WEIGHT_RECORD,
        &registrar_pda.to_bytes(),
        &payer.pubkey().to_bytes(),
    ];
    let (voter_weight_record, voter_weight_record_bump) =
        Pubkey::find_program_address(seeds, &program::id());

    action::create_voter(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &registrar_pda,
        &voter_pda,
        voter_bump,
        &voter_weight_record,
        voter_weight_record_bump,
    )
    .await
    .unwrap();

    // ------ create_deposit ------
    let voting_token_pda = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &voting_mint_a_pda,
    );
    // - mint token to votet's ATA
    spl_token_action::mint(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &mint_a.pubkey(),
        &vault_a.pubkey(),
        &payer,
        100,
    )
    .await
    .expect("mint tests");

    //voter's PDA is valid after create_ix in create_deposit
    let amount = 10;
    action::create_deposit(
        &mut banks_client,
        &payer,
        recent_blockhash,
        &registrar_pda,
        &voter_pda,
        &mint_a.pubkey(),
        &voting_mint_a_pda,
        &vault_a.pubkey(),
        &exchange_vault_a_pda,
        &voting_token_pda,
        program::state::LockupKind::Cliff,
        amount,
        2,
    )
    .await
    .unwrap();

    // ------- Assert_deposit -------

    let voter: state::Voter = banks_client
        .get_account_data_with_borsh(voter_pda)
        .await
        .unwrap();

    let idx = if let Some(idx) = voter.deposits.iter().position(|i| i.is_used == false) {
        if idx >= 1 {
            idx - 1
        } else {
            panic!("equals to 0")
        }
    } else {
        panic!("find updated deposit");
    };

    let registrar: state::Registrar = banks_client
        .get_account_data_with_borsh(registrar_pda)
        .await
        .unwrap();

    let deposit_er = voter.deposits[idx];
    let exchange_er = registrar.rates[deposit_er.rate_idx as usize];

    print!("create_deposit");
    assert_eq!(deposit_er.rate_idx, 0);
    assert_eq!(deposit_er.is_used, true);
    assert_eq!(deposit_er.amount_withdrawn, 0);

    print!("update_deposit");
    let convert_q = registrar.convert(&exchange_er, amount).unwrap();
    assert_eq!(deposit_er.amount_deposited, convert_q);
}
