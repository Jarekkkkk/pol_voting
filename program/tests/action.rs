use {
    program::instruction,
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
        id, instruction as tokenInstruction,
        state::{Account, Mint},
    },
};

//  ------- token_action -------
pub async fn create_token_and_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    owner: &Pubkey,
    mint: &Keypair,
    decimals: u8,
    vault: &Keypair,
) -> Result<(), TransportError> {
    create_mint(banks_client, payer, recent_blockhash, mint, owner, decimals).await?;
    create_token_account(
        banks_client,
        payer,
        recent_blockhash,
        vault,
        &mint.pubkey(),
        owner,
    )
    .await?;

    Ok(())
}
pub async fn create_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    pool_mint: &Keypair,
    manager: &Pubkey,
    decimals: u8,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &pool_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &id(),
            ),
            tokenInstruction::initialize_mint(&id(), &pool_mint.pubkey(), manager, None, decimals)
                .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, pool_mint],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}
pub async fn create_token_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    account: &Keypair,
    pool_mint: &Pubkey,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(Account::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                Account::LEN as u64,
                &id(),
            ),
            tokenInstruction::initialize_account(&id(), &account.pubkey(), pool_mint, owner)
                .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, account],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

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
