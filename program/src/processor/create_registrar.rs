use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    state::{ExchangeRateEntry, Registrar},
    utils::account_info::create_and_serialize_account_signed,
};
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    rate_decimals: u8,
    registrar_bump: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer_account = next_account_info(account_info_iter)?; //.0
    let authority_account = next_account_info(account_info_iter)?; //.1
    let realm_account = next_account_info(account_info_iter)?; //.2 - this is still under unchecked
    let realm_community_mint_account = next_account_info(account_info_iter)?; //.3
    let registrar_account = next_account_info(account_info_iter)?; //.4
    let _system_program = next_account_info(account_info_iter)?; //.5

    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let new_registrar = Registrar {
        authority: *authority_account.key,
        realm: *realm_account.key,
        realm_community_mint: *realm_community_mint_account.key,
        bump: registrar_bump,
        rates: [ExchangeRateEntry::default(), ExchangeRateEntry::default()],
        rate_decimals,
    };

    let seeds: &[&[_]] = &[&realm_account.key.to_bytes()];

    create_and_serialize_account_signed(
        registrar_account,
        &new_registrar,
        payer_account,
        program_id,
        seeds,
        Some(registrar_bump),
    )?;

    Ok(())
}
