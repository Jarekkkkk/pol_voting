use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::{error::TokenError, state::Account as Token};

use crate::{
    error::GovError,
    state::{Lockup, LockupKind, Registrar, Voter, SECS_PER_DAY},
};

use borsh::BorshDeserialize;

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    kind: LockupKind,
    amount: u64,
    days: i32,
) -> ProgramResult {
    msg!("foo");
    let account_info_iter = &mut accounts.iter();

    //customized
    let authority_account = next_account_info(account_info_iter)?;
    let registrar_account = next_account_info(account_info_iter)?;
    let voter_account = next_account_info(account_info_iter)?;
    //mint
    let deposit_mint_account = next_account_info(account_info_iter)?;
    let voting_mint_account = next_account_info(account_info_iter)?;
    //token
    let deposit_token_account = next_account_info(account_info_iter)?;
    let exchange_vault_account = next_account_info(account_info_iter)?;
    let voting_token_account = next_account_info(account_info_iter)?;
    //program
    let _system_program_account = next_account_info(account_info_iter)?;
    let token_program_account = next_account_info(account_info_iter)?;
    let _associated_token_account = next_account_info(account_info_iter)?;
    let _rent_account = next_account_info(account_info_iter)?;

    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let registrar: Registrar = Registrar::try_from_slice(&registrar_account.try_borrow_data()?)?;
    let mut voter: Voter = Voter::try_from_slice(&mut voter_account.try_borrow_mut_data()?)?;
    if voter.authority != *authority_account.key {
        return Err(GovError::AuthorityMismatch.into());
    }
    if voter.registrar != *registrar_account.key {
        return Err(GovError::RegistrarMismatch.into());
    }
    let voting_mint_seeds: &[&[_]] = &[
        &registrar_account.key.to_bytes(),
        &deposit_mint_account.key.to_bytes(),
    ];
    let (voting_mint_pda, _bump) = Pubkey::find_program_address(voting_mint_seeds, program_id);
    if voting_mint_account.key != &voting_mint_pda {
        return Err(ProgramError::InvalidSeeds);
    }
    let deposit_token: Token = Token::unpack(&deposit_token_account.try_borrow_data()?)?;
    if deposit_token.mint != *deposit_mint_account.key {
        return Err(TokenError::MintMismatch.into());
    }
    //Wh no need to check the ATA,
    //since it is also the PDA as well
    let exchange_vault: Token = Token::unpack(&exchange_vault_account.try_borrow_data()?)?;
    if exchange_vault.owner != *registrar_account.key {
        return Err(TokenError::OwnerMismatch.into());
    }
    if exchange_vault.mint != *deposit_mint_account.key {
        return Err(TokenError::MintMismatch.into());
    }

    //add ifelse statement to create it when non-exist, currently assume it is created
    if voting_token_account.data_is_empty() {
        //PDA of single voter
        let create_voting_token_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                &authority_account.key,
                &authority_account.key,
                &voting_mint_pda,
            );
        invoke(&create_voting_token_ix, accounts)?;
        msg!("Voting token ATA created")
    }
    let voting_token = Token::unpack(&voting_token_account.try_borrow_data()?)?;
    if voting_token.owner != *authority_account.key {
        return Err(TokenError::OwnerMismatch.into());
    }
    if voting_token.mint != *voting_mint_account.key {
        return Err(TokenError::MintMismatch.into());
    };

    //Logic
    //start time of lockup
    let start_ts = Clock::get()?.unix_timestamp;

    //create the deposit for deposit_mint in accounts arguments
    let er_idx = registrar
        .rates
        .iter()
        .position(|r| r.mint == *deposit_mint_account.key)
        .ok_or(GovError::ExchangeRateEntryNotFound)?;

    //setup the first deposit entry
    let free_deposit_er_idx = voter
        .deposits
        .iter()
        .position(|i| !i.is_used)
        .ok_or(GovError::DepositEntryFull)?;
    let free_deposit_er = &mut voter.deposits[free_deposit_er_idx];
    free_deposit_er.is_used = true;
    free_deposit_er.rate_idx = er_idx as u8;
    //should deposit be set to "0" ?
    free_deposit_er.amount_withdrawn = 0;
    free_deposit_er.lockup = Lockup {
        kind,
        start_ts,
        end_ts: start_ts
            .checked_add(i64::from(days).checked_mul(SECS_PER_DAY).unwrap())
            .unwrap(),
        padding: [0_u8; 16],
    };

    // ------ Update -------
    let update_idx = free_deposit_er_idx;
    // update deposit_er in voter

    let amount_scaled = {
        let er_idx = registrar
            .rates
            .iter()
            .position(|i| i.mint == *deposit_mint_account.key)
            .ok_or(GovError::ExchangeRateEntryNotFound)?;

        let er = registrar.rates[er_idx];
        registrar.convert(&er, amount)?
    };

    if !(voter.deposits.len() > update_idx) {
        return Err(GovError::InvalidDepositId.into());
    }
    let d_er = &mut voter.deposits[update_idx];
    d_er.amount_deposited += amount; //pure deposit
    d_er.amount_deposited += amount_scaled; //converted deposit
                                            // transfer the token to the registrar ( from deposit_token into er_vault_a)

    //exceed the heap
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        deposit_token_account.key,
        exchange_vault_account.key,
        authority_account.key,
        &[&authority_account.key],
        amount,
    )?;

    invoke(
        &transfer_ix,
        &[
            token_program_account.clone(),
            deposit_token_account.clone(),
            exchange_vault_account.clone(),
            authority_account.clone(),
        ],
    )?;
    msg!("transfer into exchange vault");

    // thawn the voting_token account if it is frozen by the authority of `registrar`
    // When will the account be frozen ???
    if voting_token.is_frozen() {
        let thaw_ix = spl_token::instruction::thaw_account(
            &spl_token::id(),
            voting_token_account.key,
            voting_mint_account.key,
            registrar_account.key,
            &[registrar_account.key],
        )?;
        invoke_signed(
            &thaw_ix,
            &[
                token_program_account.clone(),
                voting_token_account.clone(),
                voting_mint_account.clone(),
                registrar_account.clone(),
            ],
            &[&[registrar.realm.as_ref(), &[registrar.bump]]],
        )?;
        msg!("thaw voting token account");
    }

    // mint the voting_token to depositor
    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        voting_mint_account.key,
        voting_token_account.key,
        registrar_account.key,
        &[registrar_account.key],
        amount,
    )?;
    invoke_signed(
        &mint_ix,
        &[
            token_program_account.clone(),
            voting_token_account.clone(),
            voting_mint_account.clone(),
            registrar_account.clone(),
        ],
        &[&[registrar.realm.as_ref(), &[registrar.bump]]],
    )?;
    msg!("mint '{}' voting token account", &amount);

    //frozen the voting_token
    let freeze_ix = spl_token::instruction::freeze_account(
        &spl_token::id(),
        voting_token_account.key,
        voting_mint_account.key,
        registrar_account.key,
        &[registrar_account.key],
    )?;
    invoke_signed(
        &freeze_ix,
        &[
            token_program_account.clone(),
            voting_token_account.clone(),
            voting_mint_account.clone(),
            registrar_account.clone(),
        ],
        &[&[registrar.realm.as_ref(), &[registrar.bump]]],
    )?;
    msg!("freeze voting token account");

    Ok(())
}
