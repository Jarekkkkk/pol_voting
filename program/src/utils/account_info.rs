use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, system_instruction, sysvar::Sysvar,
};

use borsh::BorshSerialize;

//extend from default
pub trait Acc: PartialEq + Default {
    ///[Default] trait
    fn default(&self) -> Self;

    ///return `None` if created acconut following the input account_data instance
    /// ```rust,no_run
    /// let (serialized_data, size) = if let Some(max_size) = account_data.get_max_size() {
    ///     (None, max_size)
    /// } else {
    ///     let data = account_data.default().try_to_vec()?;
    ///     let size = data.len();
    ///     (Some(data), size)
    /// };
    /// ```
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

///Create account whose owner is sol_program
pub fn create_and_serialize_account<'a, T: BorshSerialize + Acc>(
    //`lifetime` should be applied to conform to the CPI calling
    account_info: &AccountInfo<'a>,
    account_data: &T,
    payer_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    if !account_info.data_is_empty() && account_info.owner == program_id {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (serialized_data, size) = if let Some(max_size) = account_data.get_max_size() {
        (None, max_size)
    } else {
        let data = account_data.default().try_to_vec()?;
        let size = data.len();
        (Some(data), size)
    };

    let ix = system_instruction::create_account(
        payer_info.key,
        account_info.key,
        Rent::get()?.minimum_balance(size),
        size as u64,
        program_id,
    );

    invoke(&ix, &[payer_info.clone(), account_info.clone()])?;

    msg!("PDA created");

    if let Some(serialized_data) = serialized_data {
        //full usage of account data
        account_info
            .try_borrow_mut_data()?
            .copy_from_slice(&serialized_data);
    } else {
        //leave empty bytes for upgradable or discriminator
        account_data.serialize(&mut *account_info.try_borrow_mut_data()?)?;
    }

    Ok(())
}

pub fn create_and_serialize_account_signed<'a, T: BorshSerialize + Default + PartialEq>(
    account_info: &AccountInfo<'a>,
    account_data: &T,
    payer_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    //verify PDA

    // serialized_data + size

    //seeds

    // reroute the ix by whether lamports is positive
    Ok(())
}
