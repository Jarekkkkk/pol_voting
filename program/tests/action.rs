use {
    program::instruction,
    solana_program_test::BanksClient,
    solana_sdk::{
        hash::Hash,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
        transport::TransportError,
    },
};

//  ------- program_action -------
pub async fn create_registrar(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    realm: &Pubkey,
    authority: &Pubkey,
    community_mint_pubkey: &Pubkey,
    registrar_pda: Pubkey,
    registrar_bump: u8,
    rate_decimals: u8,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::create_registrar(
            &payer.pubkey(),
            authority,
            realm,
            community_mint_pubkey,
            rate_decimals,
            &registrar_pda,
            registrar_bump,
        )],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;

    Ok(())
}

pub async fn create_exchange_rate(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    authority: &Pubkey,
    registrar_pda: &Pubkey,
    deposit_mint: &Pubkey,
    exchange_vault_pda: &Pubkey,
    voting_mint_pda: &Pubkey,
    voting_mint_bump: u8,
    idx: u16,
    er: program::state::ExchangeRateEntry,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::create_exchange_rate(
            authority,
            registrar_pda,
            deposit_mint,
            exchange_vault_pda,
            voting_mint_pda,
            voting_mint_bump,
            idx,
            er,
        )],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;

    Ok(())
}

pub async fn create_voter(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    registrar_pda: &Pubkey,
    voter_pda: &Pubkey,
    voter_bump: u8,
    voter_weight_record: &Pubkey,
    voter_weight_record_bump: u8,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::create_voter(
            &payer.pubkey(),
            &payer.pubkey(),
            registrar_pda,
            voter_pda,
            voter_bump,
            voter_weight_record,
            voter_weight_record_bump,
        )],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await?;

    Ok(())
}

pub async fn create_deposit(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    registrar_pda: &Pubkey,
    voter_pda: &Pubkey,
    deposit_mint: &Pubkey,
    voting_mint_pda: &Pubkey,
    deposit_token: &Pubkey,
    exchange_vault_pda: &Pubkey,
    voting_token: &Pubkey,
    kind: program::state::LockupKind,
    amount: u64,
    days: i32,
) -> Result<(), TransportError> {
    let tx = Transaction::new_signed_with_payer(
        &[instruction::create_deposit(
            &payer.pubkey(),
            registrar_pda,
            voter_pda,
            deposit_mint,
            voting_mint_pda,
            deposit_token,
            exchange_vault_pda,
            voting_token,
            kind,
            amount,
            days,
        )],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn update_deposit(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    registrar_pda: &Pubkey,
    voter_pda: &Pubkey,
    deposit_mint: &Pubkey,
    voting_mint_pda: &Pubkey,
    deposit_token: &Pubkey,
    exchange_vault_pda: &Pubkey,
    voting_token: &Pubkey,
    update_idx: u8,
    amount: u64,
) -> Result<(), TransportError> {
    let tx = Transaction::new_signed_with_payer(
        &[instruction::update_deposit(
            &payer.pubkey(),
            registrar_pda,
            voter_pda,
            deposit_mint,
            voting_mint_pda,
            deposit_token,
            exchange_vault_pda,
            voting_token,
            update_idx,
            amount,
        )],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await?;

    Ok(())
}
