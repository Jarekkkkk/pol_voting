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
        instruction as tokenInstruction,
        state::{Account, Mint},
    },
};

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

///require mint become signer
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
                &spl_token::id(),
            ),
            tokenInstruction::initialize_mint(
                &spl_token::id(),
                &pool_mint.pubkey(),
                manager,
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, pool_mint], //for system_ix::create_account require signature of "from + to"
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

///require token_account become signer
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
                &spl_token::id(),
            ),
            tokenInstruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                pool_mint,
                owner,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, account],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn mint(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    mint: &Pubkey,
    destination: &Pubkey,
    mint_authority: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let tx = Transaction::new_signed_with_payer(
        &[tokenInstruction::mint_to(
            &spl_token::id(),
            mint,
            destination,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .expect("mint action")],
        Some(&payer.pubkey()), //is it possible to seperate payer and signer (?
        &[mint_authority],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await?;

    Ok(())
}

pub async fn transfer(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    source: &Pubkey,
    destination: &Pubkey,
    source_owner: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[tokenInstruction::transfer(
            &spl_token::id(),
            source,
            destination,
            &source_owner.pubkey(),
            &[],
            amount,
        )
        .expect("spl_token transfer ix")],
        Some(&payer.pubkey()),
        &[source_owner],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await?;
    Ok(())
}
