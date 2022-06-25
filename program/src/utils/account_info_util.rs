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

use borsh::{try_from_slice_with_schema, BorshDeserialize, BorshSerialize};

//extend the initialize trait that could be seperated from Default value
// Default -> Invalid Value
// Initialzied -> Valid Pattern but wihtout any further modification
pub trait Acc {
    ///return `None` if created acconut following the input account_data instance
    fn get_max_size(&self) -> Option<usize> {
        None
    }

    ///limited to our program, since we have no right to overwritting data on PDA whose owner is other program
    fn verify_pda(seeds: &[&[u8]], account: &Pubkey) -> ProgramResult {
        let pda = Pubkey::find_program_address(seeds, &crate::id()).0;
        if pda != *account {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(())
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

///Create PDA whose lamports might be positive
/// Question: Hot to solve the dynamic vector of AccountInfo for doing CPI
pub fn create_and_serialize_account_signed<'a, T: BorshSerialize + Acc + PartialEq>(
    account_info: &AccountInfo<'a>,
    account_data: &T,
    payer_info: &AccountInfo<'a>,
    owner: &Pubkey,
    seeds: &[&[u8]],
    bump: Option<u8>, // whether created by custodial bump
) -> ProgramResult {
    //verify PDA
    let (pda, bump) = if let Some(bump) = bump {
        (*account_info.key, bump)
    } else {
        Pubkey::find_program_address(seeds, owner)
    };

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

        let ix = system_instruction::assign(account_info.key, owner);
        invoke_signed(&ix, &[account_info.clone()], &[&signer_seeds[..]])?;

        msg!("PDA upgraded");
    } else {
        let ix = system_instruction::create_account(
            payer_info.key,
            account_info.key,
            Rent::get()?.minimum_balance(size),
            size as u64,
            owner,
        );
        invoke_signed(
            &ix,
            &[payer_info.clone(), account_info.clone()],
            &[&signer_seeds[..]],
        )?;
        msg!("PDA created")
    }

    // === this should be seperated into 2 function ===, as we want update the state after creating account

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

///readonly account check and seralized
///
pub fn assert_and_serialized_readonly<T>(account: &AccountInfo, account_type: &T) -> ProgramResult
where
    T: BorshDeserialize,
{
    if account.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }
    if account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    let serialized_data: T = BorshDeserialize::try_from_slice(&account.try_borrow_data()?)?;

    Ok(())
}
