use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::utils::spl_token_util::create_and_initialize_mint;
use crate::{
    error::GovError,
    state::{ExchangeRateEntry, Registrar},
};
use borsh::{BorshDeserialize, BorshSerialize};
use spl_associated_token_account::instruction as ata_instruction;
use std::ops::Not;

use spl_token::state::Mint;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    voting_mint_bump: u8,
    idx: u16,
    er: ExchangeRateEntry, // use new_with_borsh to create ix by seriazling the obj with borsh
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let authority_account = next_account_info(account_info_iter)?; //.0
    let registrar_account = next_account_info(account_info_iter)?; //.1
    let deposit_mint_account = next_account_info(account_info_iter)?; //.2
    let exchange_vault_account = next_account_info(account_info_iter)?; //.3
    let voting_mint_account = next_account_info(account_info_iter)?; //.4
    let token_program_account = next_account_info(account_info_iter)?; //.5
    let _system_program_account = next_account_info(account_info_iter)?; //.6
    let _associated_token_program_account = next_account_info(account_info_iter)?; //.7
    let rent_info = next_account_info(account_info_iter)?; //.8

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //Verify
    let mut registrar =
        Registrar::check_and_get_mut_registrar(registrar_account, authority_account)?;

    // Creat exchange_vault as PDA
    if !exchange_vault_account.data_is_empty() {
        Err(ProgramError::AccountAlreadyInitialized)?
    }
    invoke(
        &ata_instruction::create_associated_token_account(
            &authority_account.key,
            registrar_account.key,
            &deposit_mint_account.key,
        ),
        accounts,
    )?;
    msg!("ExchangeVault ATA created");

    // voting_mint PDA
    let seeds: &[&[_]] = &[
        &registrar_account.key.to_bytes(),
        &deposit_mint_account.key.to_bytes(),
    ];
    let deposit_mint = spl_token::state::Mint::unpack(&deposit_mint_account.data.borrow())?;
    create_and_initialize_mint(
        authority_account,
        voting_mint_account,
        seeds,
        voting_mint_bump,
        registrar_account.key,
        deposit_mint.decimals,
        token_program_account,
        rent_info,
    )?;

    //logic
    if (er.rate > 0).not() {
        return Err(GovError::InvalidRate.into());
    };
    registrar.rates[idx as usize] = er;

    //seriazlie
    registrar.serialize(&mut *registrar_account.try_borrow_mut_data()?)?;

    Ok(())
}
