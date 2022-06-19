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

use crate::{
    state::{DepositEntry, Registrar, Voter},
    utils::account_info::create_and_serialize_account_signed,
};

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

    let registrar = Registrar::check_and_get_immut_registrar(registrar_account, authority_account)?;

    // state
    let new_voter = Voter {
        authority: *authority_account.key,
        registrar: *registrar_account.key,
        voter_bump,
        voter_weight_record_bump,
        deposits: [DepositEntry::default(); 10],
    };

    //pda seeds
    let voter_seeds: &[&[_]] = &[
        &registrar_account.key.to_bytes(),
        &authority_account.key.to_bytes(),
    ];

    //Why below payers are different when creating PDA ??
    create_and_serialize_account_signed(
        voter_account,
        &new_voter,
        authority_account,
        program_id,
        voter_seeds,
        Some(voter_bump),
    )?;

    // ------ voter_weight ------

    //owner shuold become program_id
    let new_voter_weight_record = VoterWeightRecord {
        account_discriminator: VoterWeightRecord::ACCOUNT_DISCRIMINATOR,
        //does discriminator would interact with Anchor program (?)
        realm: registrar.realm,
        governing_token_mint: registrar.realm_community_mint,
        governing_token_owner: *authority_account.key,
        voter_weight: 0,
        voter_weight_expiry: None,
        weight_action: None,
        weight_action_target: None,
        reserved: [0; 8],
    };

    let voter_weight_record_seeds: &[&[_]] = &[
        VOTER_WEIGHT_RECORD.as_ref(),
        &registrar_account.key.to_bytes(),
        &authority_account.key.to_bytes(),
    ];

    create_and_serialize_account_signed(
        voter_weight_record_account,
        &new_voter_weight_record,
        payer_account,
        program_id,
        voter_weight_record_seeds,
        Some(voter_weight_record_bump),
    )?;

    Ok(())
}
