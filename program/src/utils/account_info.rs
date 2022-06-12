use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};

use borsh::BorshSerialize;

///`lifetime` should be applied to conform to the CPI calling
pub fn create_and_serialize_account<'a, T: BorshSerialize + Default + PartialEq>(
    account_info: &AccountInfo<'a>,
    account_data: &T,
    payer_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    //verify
    if !account_info.data_is_empty() && account_info.owner == program_id {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let default: T = Default::default();
    let (serialized_data, size) = if account_data == &default {
        let data = default.try_to_vec()?;
        let size = data.len();
        (data, size)
    } else {
        let data = account_data.try_to_vec()?;
        let size = data.len();
        (data, size)
    };

    //ix
    let ix = system_instruction::create_account(
        payer_info.key,
        account_info.key,
        Rent::get()?.minimum_balance(size),
        size as u64,
        program_id,
    );

    //CPI
    invoke(&ix, &[payer_info.clone(), account_info.clone()])?;

    //msg
    msg!("PDA created");

    //serialize
    account_info
        .try_borrow_mut_data()?
        .copy_from_slice(&serialized_data);

    Ok(())
}

pub fn create_and_serialize_account_signed<'a, T: BorshSerialize + Default + PartialEq>(
    account_info: &AccountInfo<'a>,
    account_data: &T,
    payer_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    Ok(())
}
