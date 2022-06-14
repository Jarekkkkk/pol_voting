use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::state::{DepositEntry, Registrar, Voter};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_governance_addin_api::voter_weight::VoterWeightRecord;

pub const VOTER_WEIGHT_RECORD: [u8; 19] = *b"voter-weight-record";

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    voter_bump: u8,
    voter_weight_record_bump: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;
    let registrar_account = next_account_info(account_info_iter)?;
    let voter_account = next_account_info(account_info_iter)?;
    let voter_weight_record_account = next_account_info(account_info_iter)?;
    let _system_program_account = next_account_info(account_info_iter)?;
    let _token_program_account = next_account_info(account_info_iter)?;
    let _associated_token_account = next_account_info(account_info_iter)?;
    let _rent_account = next_account_info(account_info_iter)?;

    if !payer_account.is_signer && !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    //no need of checking authority for it is readonly
    if registrar_account.data_is_empty() || registrar_account.is_writable {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let registrar: Registrar = Registrar::try_from_slice(&registrar_account.try_borrow_data()?)?;

    // state
    let new_voter = Voter {
        authority: *authority_account.key,
        registrar: *registrar_account.key,
        voter_bump,
        voter_weight_record_bump,
        deposits: [DepositEntry::default(); 10],
    };
    let new_voter_serialized = new_voter.try_to_vec()?;
    let new_voter_serialized_len = new_voter_serialized.len();

    //owner shuold become program_id
    //discriminator would interact with Anchor program (?)
    let new_voter_weight_record = VoterWeightRecord {
        account_discriminator: VoterWeightRecord::ACCOUNT_DISCRIMINATOR,
        realm: registrar.realm,
        governing_token_mint: registrar.realm_community_mint,
        governing_token_owner: *authority_account.key,
        voter_weight: 0,
        voter_weight_expiry: None,
        weight_action: None,
        weight_action_target: None,
        reserved: [0; 8],
    };
    let new_voter_weight_record_serialized = new_voter_weight_record.try_to_vec()?;
    let new_voter_weight_record_serialized_len = new_voter_weight_record_serialized.len();

    //pda seeds
    let voter_seeds: &[&[_]] = &[
        &registrar_account.key.to_bytes(),
        &authority_account.key.to_bytes(),
        &[voter_bump],
    ];
    let voter_weight_record_seeds: &[&[_]] = &[
        VOTER_WEIGHT_RECORD.as_ref(),
        &registrar_account.key.to_bytes(),
        &authority_account.key.to_bytes(),
        &[voter_weight_record_bump],
    ];

    //ix
    // Why below payers are different when creating PDA ??
    let create_voter_ix = system_instruction::create_account(
        authority_account.key,
        voter_account.key,
        Rent::get()?.minimum_balance(new_voter_serialized_len),
        new_voter_serialized_len as u64,
        program_id,
    );
    msg!("create voter ix done");

    invoke_signed(
        &create_voter_ix,
        &[authority_account.clone(), voter_account.clone()],
        &[voter_seeds],
    )?;
    msg!("Voter PDA created");

    voter_account
        .try_borrow_mut_data()?
        .copy_from_slice(&new_voter_serialized);
    msg!("Voter PDA initialized");

    let create_voter_weight_record_ix = system_instruction::create_account(
        payer_account.key,
        voter_weight_record_account.key,
        Rent::get()?.minimum_balance(new_voter_weight_record_serialized_len),
        new_voter_weight_record_serialized_len as u64,
        program_id,
    );
    invoke_signed(
        &create_voter_weight_record_ix,
        &[payer_account.clone(), voter_weight_record_account.clone()],
        &[voter_weight_record_seeds],
    )?;
    msg!("Voter_weight_record PDA created");

    voter_weight_record_account
        .try_borrow_mut_data()?
        .copy_from_slice(&new_voter_weight_record_serialized);
    msg!("Voter_weight_record PDA initialized");

    Ok(())
}
