#![allow(clippy::use_self)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    error::*,
    state::{ExchangeRateEntry, LockupKind},
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum GovInstruction {
    ///Createa a new voting registrar. There can only be single registrar per governance realm.
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[writable;Signer]` payer<AccountIcreate_accountnfo>
    /// 1. `[readonly;Signer]` authority<AccountInfo>
    /// 2. `[readonly]` realm<Account>
    /// 3. `[readonly]` realm_community_mint<spl_token::Mint>
    /// 4. `[writable;PDA]` registrar<AccountInfo
    /// 5. `[readonly]` system_program
    CreateRegistrar {
        rate_decimals: u8,
        registrar_bump: u8,
    },
    /// Creates a new exchange rate for a given mint.
    /// Calculated by vault.  This allows a voter to
    /// deposit the mint in exchange for vTokens.
    /// There can only be a single exchange rate per mint
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` authority<AccountInfo>
    /// 1. `[writable]` registrar<Registrar>
    /// 2. `[readonly]` depositMint<Mint>
    /// 3. `[writable; PDA]` exchangeVault<ATA; ExchangeVault>
    /// 4. `[writable; PDA]` votingMint<Mint>
    /// 5. `[]` token_program
    /// 6. `[]` system_program_acc
    /// 7. `[]` associated_token_program
    /// 8. `[sysvar]` rent: required when invoking token program
    CreateExchangeRate {
        voting_mint_bump: u8,
        idx: u16,
        er: ExchangeRateEntry,
    },
    /// Create and init 2 PDA (voter & voter_weight_record)
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` payer
    /// 1. `[signer]` authority
    /// 2. `[readonly]` registrar
    /// 3. `[writable; PDA]` voter<Voter>
    /// 4. `[writable; PDA]` voter_weight_record<VoterWeightRecord>
    /// 5. `[]` system_program_acc
    /// 6. `[]` token_program
    /// 7. `[]` associated_token_program
    /// 8. `[sysvar]` rent
    CreateVoter {
        voter_bump: u8,
        voter_weight_record_bump: u8,
    },
    /// Creates a new DepositEntry and updates it by transferring in tokens
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` authority
    /// 1. `[readable; PDA]` regitrar
    /// 2. `[writable; PDA]` voter<Voter>
    /// 3. `[readony]` deposit_mint<Mint>
    /// 4. `[readonly]` voting_mint<Mint>
    /// 5. `[writable]` deposit_token<Token>  valut of main progrm
    /// 6. `[writable]` exchange_vault<ATA>
    /// 7. `[writable]` voting_token<ATA>
    /// 8. `[]` system_program
    /// 9. `[]` token_program
    /// 10. `[]` associated_token_program
    /// 11. `[sysvar]` rent
    CreateDeposit {
        kind: LockupKind,
        amount: u64,
        days: i32,
    },
    UpdateDeposit {
        update_idx: u8,
        amount: u64,
    },
}

impl GovInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, GovError> {
        Self::try_from_slice(input).map_err(|error| {
            msg!(&error.to_string());
            GovError::InvalidInstruction.into()
        })
    }
}

//helper functions to build up the instructions externally
pub fn create_registrar(
    payer: &Pubkey,
    authority: &Pubkey,
    realm: &Pubkey,
    community_mint: &Pubkey,
    rate_decimals: u8,
    registrar_pda: &Pubkey,
    registrar_bump: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(*realm, false),
        AccountMeta::new_readonly(*community_mint, false),
        AccountMeta::new(*registrar_pda, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction::new_with_borsh(
        crate::id(),
        &GovInstruction::CreateRegistrar {
            rate_decimals,
            registrar_bump,
        },
        accounts,
    )
}

pub fn create_exchange_rate(
    authority: &Pubkey,
    registrar_pda: &Pubkey,
    deposit_mint: &Pubkey,
    exchange_vault_pda: &Pubkey,
    voting_mint_pda: &Pubkey,
    voting_mint_bump: u8,
    idx: u16,
    er: ExchangeRateEntry,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*registrar_pda, false),
        AccountMeta::new_readonly(*deposit_mint, false),
        AccountMeta::new(*exchange_vault_pda, false), //PDA + ATA of depositMint
        AccountMeta::new(*voting_mint_pda, false),    //PDA to become Mint
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        crate::id(),
        &GovInstruction::CreateExchangeRate {
            voting_mint_bump,
            idx,
            er,
        },
        accounts,
    )
}

pub fn create_voter(
    payer: &Pubkey,
    authority: &Pubkey,
    registrar_pda: &Pubkey,
    voter_pda: &Pubkey,
    voter_bump: u8,
    voter_weight_record: &Pubkey,
    voter_weight_record_bump: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(*registrar_pda, false),
        AccountMeta::new(*voter_pda, false),
        AccountMeta::new(*voter_weight_record, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        crate::id(),
        &GovInstruction::CreateVoter {
            voter_bump,
            voter_weight_record_bump,
        },
        accounts,
    )
}

pub fn create_deposit(
    authority: &Pubkey,
    registrar_pda: &Pubkey,
    voter: &Pubkey,
    deposit_mint: &Pubkey,
    voting_mint_pda: &Pubkey,
    deposit_token: &Pubkey,
    exchange_vault_pda: &Pubkey,
    voting_token: &Pubkey,
    kind: LockupKind,
    amount: u64,
    days: i32,
) -> Instruction {
    //notice that some accounts reuqire become writable for CPI invoke
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(*registrar_pda, false),
        AccountMeta::new(*voter, false),
        AccountMeta::new_readonly(*deposit_mint, false),
        AccountMeta::new(*voting_mint_pda, false), /*voting_mint require writable as we will mint for CPI */
        AccountMeta::new(*deposit_token, false),
        AccountMeta::new(*exchange_vault_pda, false),
        //PDA be created
        AccountMeta::new(*voting_token, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        crate::id(),
        &GovInstruction::CreateDeposit { kind, amount, days },
        accounts,
    )
}

pub fn update_deposit(
    authority: &Pubkey,
    registrar_pda: &Pubkey,
    voter: &Pubkey,
    deposit_mint: &Pubkey,
    voting_mint_pda: &Pubkey,
    deposit_token: &Pubkey,
    exchange_vault_pda: &Pubkey,
    voting_token: &Pubkey,
    update_idx: u8,
    amount: u64,
) -> Instruction {
    //notice that some accounts reuqire become writable for CPI invoke
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new_readonly(*registrar_pda, false),
        AccountMeta::new(*voter, false),
        AccountMeta::new_readonly(*deposit_mint, false),
        AccountMeta::new_readonly(*voting_mint_pda, false),
        AccountMeta::new(*deposit_token, false),
        AccountMeta::new(*exchange_vault_pda, false),
        //PDA be created
        AccountMeta::new(*voting_token, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction::new_with_borsh(
        crate::id(),
        &GovInstruction::UpdateDeposit { update_idx, amount },
        accounts,
    )
}
