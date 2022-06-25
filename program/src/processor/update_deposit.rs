use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::{error::TokenError, state::Account as Token};

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    error::GovError,
    state::{DepositEntry, Lockup, LockupKind, Registrar, Voter, SECS_PER_DAY},
    utils::spl_token_util,
};

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    update_idx: u8,
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    //customized
    let authority_info = next_account_info(account_info_iter)?;
    let registrar_info = next_account_info(account_info_iter)?;
    let voter_info = next_account_info(account_info_iter)?;
    //mint
    let deposit_mint_info = next_account_info(account_info_iter)?;
    let voting_mint_info = next_account_info(account_info_iter)?;
    //token
    let deposit_token_info = next_account_info(account_info_iter)?;
    let exchange_vault_info = next_account_info(account_info_iter)?;
    let voting_token_info = next_account_info(account_info_iter)?;
    //program
    let system_program_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let _associated_token_info = next_account_info(account_info_iter)?;
    let _rent_info = next_account_info(account_info_iter)?;

    //unpack

    let registrar: Registrar = Registrar::try_from_slice(&registrar_info.try_borrow_data()?)?;
    msg!("Unable to deseriazlie");
    let mut voter: Voter = Voter::try_from_slice(&voter_info.try_borrow_mut_data()?)?;

    DepositEntry::update_deposit(
        &mut voter,
        &registrar,
        update_idx,
        amount,
        voter_info,
        deposit_mint_info,
    )?;

    //transfer token A from {voter} to {exchange_vault}
    spl_token_util::transfer_spl_token(
        deposit_token_info,
        exchange_vault_info,
        authority_info,
        amount,
        token_program_info,
    )?;
    //mint governance token
    msg!("mint voting tokne");
    let seeds: &[&[_]] = &[&registrar.realm.to_bytes()];
    spl_token_util::mint_token_signed(
        voting_token_info,
        voting_mint_info,
        registrar_info,
        seeds,
        registrar.bump,
        amount,
        token_program_info,
        "voting_token",
    )?;

    Ok(())
}
