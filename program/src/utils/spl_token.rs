use solana_program::{
    account_info::AccountInfo,
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

use spl_token::state::Account;

// ------- create_account -------

fn create_token_account<F>(account: &AccountInfo, check: F) -> ProgramResult {
    todo!()
}

///[spl_token] program create and assign authority without passing `token_account`
pub fn create_and_initialize_mint<'a>(
    payer: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    seeds: &[&[u8]],
    bump: u8,
    mint_authority: &Pubkey,
    decimals: u8,
    spl_token_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
) -> ProgramResult {
    //const: [owner] of token account will be spl_token_program
    //verify
    if !account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    //deserialize & required declaration
    let size = spl_token::state::Mint::get_packed_len();
    let mut signer_seeds = seeds.to_vec();
    let bump = &[bump];
    signer_seeds.push(bump);

    //.0    main logic (ix, CPI, state update)
    let ix = system_instruction::create_account(
        payer.key,
        account.key,
        Rent::get()?.minimum_balance(size),
        size as u64,
        &spl_token::id(), //owner be fixed as spl_token
    );

    invoke_signed(&ix, &[payer.clone(), account.clone()], &[&signer_seeds[..]])?;
    msg!("PDA in mint size created");
    //.1    init Mint
    let init_vm_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        account.key,
        mint_authority,
        Some(mint_authority),
        decimals,
    )?;
    //required signature: `mint_authority`
    invoke_signed(
        &init_vm_mint_ix,
        &[account.clone(), rent.clone(), spl_token_program.clone()],
        &[&signer_seeds[..]],
    )?;
    msg!("Mint initialized");

    // seriazlied
    // no need

    Ok(())
}

// ------- token_action -------

///basic transfer with `invoke`
pub fn transfer_spl_token<'a>(
    source_account: &AccountInfo<'a>,
    destination_account: &AccountInfo<'a>,
    source_owner: &AccountInfo<'a>,
    amount: u64,
    spl_token_program: &AccountInfo<'a>,
) -> ProgramResult {
    msg!("transfer spl_token");
    invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            source_account.key,
            destination_account.key,
            source_owner.key,
            &[&source_owner.key],
            amount,
        )?,
        &[
            spl_token_program.clone(),
            source_account.clone(),
            destination_account.clone(),
            source_owner.clone(),
        ],
    )?;
    msg!(
        "transfer amount:{} to spl_token {}",
        &amount,
        destination_account.key
    );

    Ok(())
}
pub fn transfer_spl_token_signed() {
    todo!();
}

/// check_freeze, mint, and freeze back with `invoke_signed`
pub fn mint_token_signed<'a>(
    destination_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    token_owner: &AccountInfo<'a>,
    seeds: &[&[u8]],
    bump: u8,
    amount: u64,
    token_program: &AccountInfo<'a>,
    name: &str,
) -> ProgramResult {
    let token: Account = Account::unpack(&destination_account.try_borrow_data()?)?;

    let mut signer_seeds = seeds.to_vec();
    let bump = &[bump];
    signer_seeds.push(bump);

    if token.is_frozen() {
        invoke_signed(
            &spl_token::instruction::thaw_account(
                &spl_token::id(),
                destination_account.key,
                mint.key,
                token_owner.key,
                &[token_owner.key],
            )?,
            &[
                token_program.clone(),
                destination_account.clone(),
                mint.clone(),
                token_owner.clone(),
            ],
            &[&signer_seeds[..]],
        )?;
        msg!("thaw {} token account", name);
    }

    //mint

    invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            mint.key,
            destination_account.key,
            token_owner.key,
            &[token_owner.key],
            amount,
        )?,
        &[
            token_program.clone(),
            destination_account.clone(),
            mint.clone(),
            token_owner.clone(),
        ],
        &[&signer_seeds[..]],
    )?;
    msg!("mint '{}' to {} token account", amount, name);

    //freeze
    invoke_signed(
        &spl_token::instruction::freeze_account(
            &spl_token::id(),
            destination_account.key,
            mint.key,
            token_owner.key,
            &[token_owner.key],
        )?,
        &[
            token_program.clone(),
            destination_account.clone(),
            mint.clone(),
            token_owner.clone(),
        ],
        &[&signer_seeds[..]],
    )?;
    msg!("freeze {} token account", name);

    Ok(())
}
