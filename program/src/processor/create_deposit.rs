use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use spl_token::{error::TokenError, state::Account as Token};

use crate::{
    error::GovError,
    state::{Lockup, LockupKind, Registrar, Voter, SECS_PER_DAY},
    utils::account_info::Acc,
};

use borsh::{BorshDeserialize, BorshSerialize};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    kind: LockupKind,
    amount: u64,
    days: i32,
) -> ProgramResult {
    let update_idx = {
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?; //.0
        let registrar_account = next_account_info(account_info_iter)?; //.1
        let voter_account = next_account_info(account_info_iter)?; //.2
                                                                   //mint
        let deposit_mint_account = next_account_info(account_info_iter)?; //.3
        let voting_mint_account = next_account_info(account_info_iter)?; //.4
                                                                         //token
        let deposit_token_account = next_account_info(account_info_iter)?; //.5
        let exchange_vault_account = next_account_info(account_info_iter)?; //.6
        let voting_token_account = next_account_info(account_info_iter)?; //.7
                                                                          //program
        let _system_program_account = next_account_info(account_info_iter)?; //.8
        let token_program_account = next_account_info(account_info_iter)?; //.9
        let _associated_token_account = next_account_info(account_info_iter)?; //.10
        let _rent_account = next_account_info(account_info_iter)?; //.11

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // either unpack in 1.) fn_main
        // or 2.) in function
        let registrar: Registrar =
            Registrar::try_from_slice(&registrar_account.try_borrow_data()?)?;
        let mut voter: Voter = Voter::try_from_slice(&mut voter_account.try_borrow_mut_data()?)?;

        voter.assert_voter(authority_account.key, registrar_account.key)?;

        let voting_mint_seeds: &[&[_]] = &[
            &registrar_account.key.to_bytes(),
            &deposit_mint_account.key.to_bytes(),
        ];
        //be moved out of trait
        Voter::verify_pda(voting_mint_seeds, voting_mint_account.key)?;

        //Token program
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
        //this could be optimized by passing `fn()->bool` into fn create_token_ix
        if voting_token_account.data_is_empty() {
            //PDA of single voter
            let create_voting_token_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &authority_account.key,
                    &authority_account.key,
                    &voting_mint_account.key,
                );
            invoke(&create_voting_token_ix, accounts)?;
            msg!("Voting token ATA created")
        }
        let voting_token = Token::unpack(&voting_token_account.try_borrow_data()?)?;
        if voting_token.owner /* not owner as we defined in program */!= *authority_account.key {
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

        //serialize
        voter.serialize(&mut *voter_account.try_borrow_mut_data()?)?;

        free_deposit_er_idx as u8
    };
    //shorten the heap size

    crate::processor::update_deposit::process(program_id, accounts, update_idx, amount)?;

    Ok(())
}

//check payer::voting_token(ATA) has created
// update deposit
// transfer from {god_a} to {exchange_vault}
