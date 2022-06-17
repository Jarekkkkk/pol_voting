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

pub trait TokenAcc {
    fn create_associated_token() -> ProgramResult;
    fn initialize_mint() -> ProgramResult;
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
    invoke(
        &init_vm_mint_ix,
        &[account.clone(), rent.clone(), spl_token_program.clone()],
    )?;
    msg!("Mint initialized");

    // seriazlied
    // no need

    Ok(())
}

//pub fn
