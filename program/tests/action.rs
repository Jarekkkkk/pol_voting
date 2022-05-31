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

///Create 2 mint and vault
pub async fn create_tokens_and_mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    //five insstructions
    // create mint, init mint, create vault, init vault, mint to vault
    let mint_a = Keypair::new();
    let vault_a = Keypair::new();
    let mint_b = Keypair::new();
    let vault_b = Keypair::new();

    create_mint(banks_client, payer, recent_blockhash, &mint_a, owner, 6).await?;
    create_mint(banks_client, payer, recent_blockhash, &mint_b, owner, 0).await?;
    create_token_account(
        banks_client,
        payer,
        recent_blockhash,
        &vault_a,
        &mint_a.pubkey(),
        owner,
    )
    .await?;
    create_token_account(
        banks_client,
        payer,
        recent_blockhash,
        &vault_b,
        &mint_b.pubkey(),
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
            instruction::initialize_mint(&id(), &pool_mint.pubkey(), manager, None, decimals)
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
            instruction::initialize_account(&id(), &account.pubkey(), pool_mint, owner).unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, account],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

//PDA
pub async fn create_registrar() -> Result<(), TransportError> {
    Ok(())
}
