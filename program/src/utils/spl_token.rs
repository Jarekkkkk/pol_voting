use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, system_program,
};
use spl_associated_token_account::instruction as ata_ix;
use spl_token_2022;

pub trait TokenAcc {
    fn create_associated_token() -> ProgramResult;
    fn initialize_mint() -> ProgramResult;
}

///[spl_associated_token_account] program create and assign authority without passing `token_account`
pub fn create_associated_token<'a>(
    token_account: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    sol_program: &AccountInfo<'a>,
    spl_token_program: &AccountInfo<'a>,
) -> ProgramResult {
    let ix = ata_ix::create_associated_token_account(payer.key, authority.key, mint.key);

    //questions:: where does progrma check the associated token
    let pda = spl_associated_token_account::get_associated_token_address(authority.key, mint.key);
    if pda != *token_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    invoke(
        &ix,
        &[
            payer.clone(),
            authority.clone(),
            mint.clone(),
            token_account.clone(),
            sol_program.clone(),
            spl_token_program.clone(),
        ],
    )?;
    Ok(())
}
