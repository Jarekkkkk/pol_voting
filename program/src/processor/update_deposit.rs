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

use crate::{
    error::GovError,
    state::{Lockup, LockupKind, Registrar, Voter, SECS_PER_DAY},
};

use borsh::BorshDeserialize;

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    update_idx: u8,
    amount: u64,
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
    let token_program_account = next_account_info(account_info_iter)?;
    // let _associated_token_account = next_account_info(account_info_iter)?;
    // let _rent_account = next_account_info(account_info_iter)?;

    //unpack
    let voting_token = Token::unpack(&voting_token_account.try_borrow_data()?)?;
    let registrar: Registrar = Registrar::try_from_slice(&registrar_account.try_borrow_data()?)?;
    let mut voter: Voter = Voter::try_from_slice(&mut voter_account.try_borrow_mut_data()?)?;

    // update deposit_er in voter
    let amount_scaled = {
        let er_idx = registrar
            .rates
            .iter()
            .position(|i| i.mint == *deposit_mint_account.key)
            .ok_or(GovError::ExchangeRateEntryNotFound)?;

        let er = registrar.rates[er_idx];
        registrar.convert(&er, amount)?
    };

    if !(voter.deposits.len() > update_idx as usize) {
        return Err(GovError::InvalidDepositId.into());
    }
    let d_er = &mut voter.deposits[update_idx as usize];
    d_er.amount_deposited += amount; //pure deposit
    d_er.amount_deposited += amount_scaled;

    //ix
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        deposit_token_account.key,
        exchange_vault_account.key,
        authority_account.key,
        &[&authority_account.key],
        amount,
    )?;

    msg!("transfer ix created");
    invoke(
        &transfer_ix,
        &[
            token_program_account.clone(),
            deposit_token_account.clone(),
            exchange_vault_account.clone(),
            authority_account.clone(),
        ],
    )?;
    msg!("transfer into exchange vault");

    // thawn the voting_token account if it is frozen by the authority of `registrar`
    // When will the account be frozen ???
    if voting_token.is_frozen() {
        let thaw_ix = spl_token::instruction::thaw_account(
            &spl_token::id(),
            voting_token_account.key,
            voting_mint_account.key,
            registrar_account.key,
            &[registrar_account.key],
        )?;
        invoke_signed(
            &thaw_ix,
            &[
                token_program_account.clone(),
                voting_token_account.clone(),
                voting_mint_account.clone(),
                registrar_account.clone(),
            ],
            &[&[registrar.realm.as_ref(), &[registrar.bump]]],
        )?;
        msg!("thaw voting token account");
    }

    // mint the voting_token to depositor
    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        voting_mint_account.key,
        voting_token_account.key,
        registrar_account.key,
        &[registrar_account.key],
        amount,
    )?;
    invoke_signed(
        &mint_ix,
        &[
            token_program_account.clone(),
            voting_token_account.clone(),
            voting_mint_account.clone(),
            registrar_account.clone(),
        ],
        &[&[registrar.realm.as_ref(), &[registrar.bump]]],
    )?;
    msg!("mint '{}' voting token account", &amount);

    //frozen the voting_token
    let freeze_ix = spl_token::instruction::freeze_account(
        &spl_token::id(),
        voting_token_account.key,
        voting_mint_account.key,
        registrar_account.key,
        &[registrar_account.key],
    )?;
    invoke_signed(
        &freeze_ix,
        &[
            token_program_account.clone(),
            voting_token_account.clone(),
            voting_mint_account.clone(),
            registrar_account.clone(),
        ],
        &[&[registrar.realm.as_ref(), &[registrar.bump]]],
    )?;
    msg!("freeze voting token account");
    Ok(())
}
