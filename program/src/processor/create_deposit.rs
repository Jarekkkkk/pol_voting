use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::{
    error::TokenError,
    state::{Account as Token, Mint},
};

use crate::{
    error::GovError,
    state::{LockupKind, Registrar, Voter},
};

use borsh::{BorshDeserialize, BorshSerialize};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    kind: LockupKind,
    amount: u64,
    days: i32,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    //customized
    let authority_account = next_account_info(account_info_iter)?;
    let registrar_account = next_account_info(account_info_iter)?;
    let voter_account = next_account_info(account_info_iter)?;
    //mint
    let deposit_mint_account = next_account_info(account_info_iter)?;
    let voting_mint_account = next_account_info(account_info_iter)?;
    //token
    let deposit_token_account = next_account_info(account_info_iter)?;
    let exchange_vault_account = next_account_info(account_info_iter)?;
    let voting_token_account = next_account_info(account_info_iter)?;
    //program
    let _system_program_account = next_account_info(account_info_iter)?;
    let _token_program_account = next_account_info(account_info_iter)?;
    let _associated_token_account = next_account_info(account_info_iter)?;
    let _rent_account = next_account_info(account_info_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let registrar: Registrar = Registrar::try_from_slice(&registrar_account.try_borrow_data()?)?;

    let mut voter: Voter = Voter::try_from_slice(&mut voter_account.try_borrow_mut_data()?)?;
    if voter.authority != *authority_account.key {
        return Err(GovError::AuthorityMismatch.into());
    }
    if voter.registrar != *registrar_account.key {
        return Err(GovError::RegistrarMismatch.into());
    }

    let deposit_mint: Mint = Mint::unpack(&deposit_mint_account.try_borrow_data()?)?;

    let mut voting_mint: Mint = Mint::unpack(&mut voting_mint_account.try_borrow_mut_data()?)?;
    let voting_mint_seeds: &[&[_]] = &[
        &registrar_account.key.to_bytes(),
        &deposit_mint_account.key.to_bytes(),
    ];
    let (voting_mint_pda, _bump) = Pubkey::find_program_address(voting_mint_seeds, program_id);
    if voting_mint_account.key != &voting_mint_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let mut deposit_token: Token =
        Token::unpack(&mut deposit_token_account.try_borrow_mut_data()?)?;
    if deposit_token.mint != *deposit_mint_account.key {
        return Err(TokenError::MintMismatch.into());
    }

    //Wh no need to check the ATA,
    //since it is also the PDA as well
    let mut exchange_vault: Token =
        Token::unpack(&mut exchange_vault_account.try_borrow_mut_data()?)?;
    if exchange_vault.owner != *registrar_account.key {
        return Err(TokenError::OwnerMismatch.into());
    }
    if exchange_vault.mint != *deposit_mint_account.key {
        return Err(TokenError::MintMismatch.into());
    }

    //add ifelse statement to create it when non-exist, currently assume it is created
    let mut voting_token = Token::unpack(&mut voting_token_account.try_borrow_mut_data()?)?;
    if voting_token.owner != *authority_account.key {
        return Err(TokenError::OwnerMismatch.into());
    }
    if voting_token.mint != *voting_mint_account.key {
        return Err(TokenError::MintMismatch.into());
    }

    //Logic
    //start time of lockup
    let start_ts = Clock::get()?.unix_timestamp;

    //create the deposit for deposit_mint in accounts arguments
    let er_idx = registrar
        .rates
        .iter()
        .position(|r| r.mint == *deposit_mint_account.key)
        .ok_or(GovError::ExchangeRateEntryNotFound);

    Ok(())
}
