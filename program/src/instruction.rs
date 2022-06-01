#![allow(clippy::use_self)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    pubkey::Pubkey,
    system_program,
    sysvar::Sysvar,
};

use crate::{error::*, state::ExchangeRateEntry};

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
    /// 1. `[readonly]` registrar<Registrar>
    /// 2. `[readonly]` depositMint<Mint>
    /// 3. `[writable; PDA]` exchangeVault<ATA; ExchangeVault>
    /// 4. `[writable; PDA]` votingMint<Mint>
    /// 5. `[]` system_program_acc
    /// 6. `[]` token_program
    /// 7. `[]` associated_token_program
    CreateExchangeRate { idx: u16, er: ExchangeRateEntry },
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
