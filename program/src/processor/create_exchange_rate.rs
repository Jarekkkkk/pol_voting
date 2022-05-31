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

use borsh::BorshDeserialize;
use spl_associated_token_account::instruction::create_associated_token_account;
use std::mem::size_of;

use crate::{error::GovError, state::*};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
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
    let associated_token_program_account = next_account_info(account_info_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if registrar_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    let registrar: Registrar = Registrar::try_from_slice(&registrar_account.try_borrow_data()?)?;
    if registrar.authority != *authority_account.key {
        let err = GovError::AuthorityMismatch;
        msg!("{}", err);
        return Err(err.into());
    }

    let deposit_mint_data = deopsit_mint_account.data.borrow();
    let deposit_mint = spl_token::state::Mint::unpack(&deposit_mint_data)?;

    if !exchange_vault_account.data_is_empty() {
        Err(ProgramError::AccountAlreadyInitialized)?
    }
    //pda is created inside the ix
    let create_ev_ix = create_associated_token_account(
        authority_account.key,
        registrar_account.key,
        deopsit_mint_account.key,
    );
    invoke(&create_ev_ix, accounts)?;
    msg!("ExchangeVault ATA created");

    if !voting_mint_account.data_is_empty() {
        Err(ProgramError::AccountAlreadyInitialized)?
    }

    //1. create voting_mint as `PDA`
    let (_pda, bump) = Pubkey::find_program_address(
        &[
            registrar_account.key.as_ref(),
            deopsit_mint_account.key.as_ref(),
            token_program_account.key.as_ref(),
        ],
        associated_token_program_account.key,
    );

    let associated_token_account_signer_seeds: &[&[_]] = &[
        registrar_account.key.as_ref(),
        deopsit_mint_account.key.as_ref(),
        token_program_account.key.as_ref(),
        &[bump],
    ];
    let create_voting_mint_ix = system_instruction::create_account(
        authority_account.key,
        voting_mint_account.key,
        Rent::get()?.minimum_balance(size_of::<spl_token::state::Mint>()),
        (size_of::<spl_token::state::Mint>()) as u64,
        registrar_account.key,
    );
    invoke_signed(
        &create_voting_mint_ix,
        accounts,
        &[associated_token_account_signer_seeds],
    )?;
    msg!("voting_mint PDA created");
    //2. init as Mint
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
    if !er.rate > 0 {
        return Err(GovError::InvalidRate.into());
    };
    // probably extend the funcionality using Pack tratit (.?
    let registrar_account_data = registrar_account.try_borrow_mut_data()?;
    let mut registrar: Registrar = Registrar::try_from_slice(&registrar_account_data)?;
    registrar.rates[idx as usize] = er;

    Ok(())
}
