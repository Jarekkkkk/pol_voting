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

    let payer_account = next_account_info(account_info_iter)?;
    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let authority_account = next_account_info(account_info_iter)?;
    let realm_account = next_account_info(account_info_iter)?; //this is still under unchecked
    let realm_community_mint_account = next_account_info(account_info_iter)?;
    let registrar_account = next_account_info(account_info_iter)?;
    if !registrar_account.data_is_empty() {
        Err(ProgramError::AccountAlreadyInitialized)?
    }
    let _system_program = next_account_info(account_info_iter)?;

    //new_state
    let new_registrar = Registrar {
        authority: *authority_account.key,
        realm: *realm_account.key,
        realm_community_mint: *realm_community_mint_account.key,
        bump: registrar_bump.clone(),
        rates: [ExchangeRateEntry::default(), ExchangeRateEntry::default()],
        rate_decimals,
    };
    let new_registrar_serialized = new_registrar.try_to_vec()?;
    let registrar_size = new_registrar_serialized.len();

    let create_registrar_ix = system_instruction::create_account(
        payer_account.key,
        registrar_account.key,
        Rent::get()?.minimum_balance(registrar_size),
        registrar_size as u64,
        program_id,
    );

    let signer_seeds: &[&[_]] = &[&realm_account.key.to_bytes(), &[registrar_bump]];
    invoke_signed(&create_registrar_ix, accounts, &[signer_seeds])?;

    msg!("registrar PDA created ");

    //udpate PDA
    registrar_account
        .try_borrow_mut_data()?
        .copy_from_slice(&new_registrar_serialized);

    msg!("Registrar PDA initialized");

    Ok(())
}
