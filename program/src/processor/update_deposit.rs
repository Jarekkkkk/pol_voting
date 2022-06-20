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
    utils::spl_token as spl_token_util,
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
    let authority_info = next_account_info(account_info_iter)?;
    let registrar_info = next_account_info(account_info_iter)?;
    let voter_account = next_account_info(account_info_iter)?;
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
    let voting_token = Token::unpack(&voting_token_info.try_borrow_data()?)?;
    let registrar: Registrar = Registrar::try_from_slice(&registrar_info.try_borrow_data()?)?;
    let mut voter: Voter = Voter::try_from_slice(&mut voter_account.try_borrow_mut_data()?)?;

    // update deposit_er in voter
    let amount_scaled = {
        let er_idx = registrar
            .rates
            .iter()
            .position(|i| i.mint == *deposit_mint_info.key)
            .ok_or(GovError::ExchangeRateEntryNotFound)?;

        let er = registrar.rates[er_idx];
        registrar.convert(&er, amount)?
    };

    //be put into impl Voter
    if !(voter.deposits.len() > update_idx as usize) {
        return Err(GovError::InvalidDepositId.into());
    }
    let d_er = &mut voter.deposits[update_idx as usize];
    d_er.amount_deposited += amount; //pure deposit
    d_er.amount_scaled += amount_scaled;

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
