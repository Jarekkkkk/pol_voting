use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use borsh::BorshSerialize;

//extend from default
pub trait Acc {
    ///return `None` if created acconut following the input account_data instance
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

///Create account whose owner is sol_program
pub fn create_and_serialize_account<'a, T: BorshSerialize + Acc + PartialEq>(
    //`lifetime` should be applied to conform to the CPI calling
    account_info: &AccountInfo<'a>,
    account_data: &T,
    payer_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    if !(account_info.data_is_empty() && account_info.owner == program_id) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (serialized_data, size) = if let Some(max_size) = account_data.get_max_size() {
        (None, max_size)
    } else {
        let data = account_data.try_to_vec()?;
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

    msg!("Account created");

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

///Create PDA tahat is allowed to repeated calling
pub fn create_and_serialize_account_signed<'a, T: BorshSerialize + Acc + PartialEq>(
    account_info: &AccountInfo<'a>,
    account_data: &T,
    payer_info: &AccountInfo<'a>,
    program_id: &Pubkey,
    seeds: &[&[u8]],
) -> ProgramResult {
    //verify PDA

    let (pda, bump) = Pubkey::find_program_address(seeds, program_id);

    if pda != *account_info.key {
        //log the message by type of PDA we are goting to create
        return Err(ProgramError::InvalidSeeds);
    }

    let (serialized_data, size) = if let Some(max_size) = account_data.get_max_size() {
        (None, max_size)
    } else {
        let data = account_data.try_to_vec()?;
        let size = data.len();
        (Some(data), size)
    };

    let mut signer_seeds = seeds.to_vec();
    let bump = &[bump];
    signer_seeds.push(bump);

    let minium_lamports = Rent::get()?.minimum_balance(size).max(1);
    if account_info.lamports() > 0 {
        let top_up_lamports = minium_lamports.saturating_sub(account_info.lamports());

        if top_up_lamports > 0 {
            let ix =
                system_instruction::transfer(payer_info.key, account_info.key, top_up_lamports);

            //does the ix require program_acc (?)
            invoke(&ix, &[payer_info.clone(), account_info.clone()])?;
        }

        let ix = system_instruction::allocate(account_info.key, size as u64);
        invoke_signed(&ix, &[account_info.clone()], &[&signer_seeds[..]])?;

        let ix = system_instruction::assign(account_info.key, program_id);
        invoke_signed(&ix, &[account_info.clone()], &[&signer_seeds[..]])?;

        msg!("PDA upgraded");
    } else {
        let ix = system_instruction::create_account(
            payer_info.key,
            account_info.key,
            Rent::get()?.minimum_balance(size),
            size as u64,
            program_id,
        );

        invoke_signed(
            &ix,
            &[payer_info.clone(), account_info.clone()],
            &[&signer_seeds[..]],
        )?;
        msg!("PDA created")
    }

    if let Some(serialized_data) = serialized_data {
        account_info
            .try_borrow_mut_data()?
            .copy_from_slice(&serialized_data);
    } else {
        account_data.serialize(&mut *account_info.try_borrow_mut_data()?)?;
    }

    msg!("PDA initialized");

    Ok(())
}
