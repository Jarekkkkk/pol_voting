use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::GovError,
    state::{ExchangeRateEntry, Registrar},
};
use std::ops::Not;

use borsh::{BorshDeserialize, BorshSerialize};
use spl_associated_token_account::instruction as ata_instruction;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    voting_mint_bump: u8,
    idx: u16,
    er: ExchangeRateEntry, // use new_with_borsh to create ix by seriazling the obj with borsh
) -> ProgramResult {
    //require exchange rate > 0

    let account_info_iter = &mut accounts.iter();

    let authority_account = next_account_info(account_info_iter)?;
    let registrar_account = next_account_info(account_info_iter)?;
    let deopsit_mint_account = next_account_info(account_info_iter)?;
    let exchange_vault_account = next_account_info(account_info_iter)?;
    let voting_mint_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    let _system_program_account = next_account_info(account_info_iter)?;
    let _associated_token_program_account = next_account_info(account_info_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if registrar_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    if registrar_account.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }
    let registrar: Registrar = Registrar::try_from_slice(&registrar_account.try_borrow_data()?)?;
    if registrar.authority != *authority_account.key {
        let err = GovError::AuthorityMismatch;
        msg!("{}", err);
        return Err(err.into());
    }

    //build PDA
    //1. exchange_vault(ATA)
    if !exchange_vault_account.data_is_empty() {
        Err(ProgramError::AccountAlreadyInitialized)?
    }
    let create_ata_ix = ata_instruction::create_associated_token_account(
        &authority_account.key,
        registrar_account.key,
        &deopsit_mint_account.key,
    );
    //ata program do our favor by creating PDA and invoke_signed
    invoke(&create_ata_ix, accounts)?;
    msg!("ExchangeVault ATA created");

    //2.1 votingMint(Mint)
    if !voting_mint_account.data_is_empty() {
        Err(ProgramError::AccountAlreadyInitialized)?
    }
    let mint_size = spl_token_2022::state::Mint::get_packed_len();
    let create_voting_mint_ix = system_instruction::create_account(
        authority_account.key,
        voting_mint_account.key,
        Rent::get()?.minimum_balance(mint_size),
        mint_size as u64,
        &spl_token::id(), //make spl owner to access the Mint data
    );
    let signer_seeds: &[&[_]] = &[
        &registrar_account.key.to_bytes(),
        &deopsit_mint_account.key.to_bytes(),
        &[voting_mint_bump],
    ];
    invoke_signed(
        &create_voting_mint_ix,
        &[authority_account.clone(), voting_mint_account.clone()],
        &[signer_seeds],
    )?;
    msg!("voting_mint PDA created");

    let deposit_mint = spl_token::state::Mint::unpack(&deopsit_mint_account.data.borrow())?;
    //2.2 init voting_mint as Mint
    let init_vm_mint_ix = spl_token::instruction::initialize_mint(
        token_program_account.key,
        voting_mint_account.key,
        registrar_account.key,
        Some(registrar_account.key),
        deposit_mint.decimals,
    )?;
    invoke(&init_vm_mint_ix, accounts)?;
    msg!("voting_mint Mint initialized");

    //logic
    if (er.rate > 0).not() {
        return Err(GovError::InvalidRate.into());
    };
    // probably extend the funcionality using Pack tratit (?
    let mut registrar_account_data = registrar_account.try_borrow_mut_data()?;
    let mut registrar: Registrar = Registrar::try_from_slice(&registrar_account_data)?;
    registrar.rates[idx as usize] = er;

    //either borsh::serialzie or core::slice::copy from slice
    registrar.serialize(&mut *registrar_account_data)?;

    Ok(())
}
