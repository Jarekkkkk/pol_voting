use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use borsh::BorshSerialize;

use crate::state::{ExchangeRateEntry, Registrar};
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    rate_decimals: u8,
    registrar_bump: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    //0.signer
    let payer_account = next_account_info(account_info_iter)?;
    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    //1.authority
    let authority_account = next_account_info(account_info_iter)?;
    //2.realm: ( unchecked yet )
    let realm_account = next_account_info(account_info_iter)?;
    //ream_community_mint
    let realm_community_mint_account = next_account_info(account_info_iter)?;
    //3.registrar
    let registrar_account = next_account_info(account_info_iter)?;
    if !registrar_account.data_is_empty() {
        Err(ProgramError::AccountAlreadyInitialized)?
    }

    //check the registrar(PDA)'s bump is legal
    let seeds = &[realm_account.key.as_ref()];
    let (registrar_pda, bump) = Pubkey::find_program_address(seeds, program_id);
    if bump != registrar_bump && registrar_pda != *registrar_account.key {
        Err(ProgramError::InvalidSeeds)?
    }

    let registar_size = Registrar::serialized_size();
    let create_registrar_ix = system_instruction::create_account(
        payer_account.key,
        registrar_account.key,
        Rent::get()?.minimum_balance(registar_size),
        registar_size as u64,
        program_id,
    );

    invoke_signed(
        &create_registrar_ix,
        accounts,
        &[seeds, &[&[registrar_bump]]],
    )?; //use invoke_signed since bothe account should on behalf of signer
    msg!("Registrar PDA created");

    //udpate PDA
    let new_registar = Registrar {
        authority: *authority_account.key,
        realm: *realm_account.key,
        realm_community_mint: *realm_community_mint_account.key,
        bump: registrar_bump.clone(),
        rates: [ExchangeRateEntry::default(), ExchangeRateEntry::default()],
        rate_decimals,
    };
    new_registar.serialize(&mut *registrar_account.try_borrow_mut_data()?)?;
    msg!("Registrar PDA initialized");

    Ok(())
}
