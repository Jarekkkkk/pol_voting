use crate::instruction::GovInstruction;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

mod create_deposit;
mod create_exchange_rate;
mod create_registrar;
mod create_voter;

#[cfg_attr(feature = "no-entrypoint", allow(dead_code))]
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match GovInstruction::unpack(instruction_data)? {
        GovInstruction::CreateRegistrar {
            rate_decimals,
            registrar_bump,
        } => {
            msg!("Instruction: create registrar");
            create_registrar::process(program_id, accounts, rate_decimals, registrar_bump)
        }
        GovInstruction::CreateExchangeRate {
            voting_mint_bump,
            idx,
            er,
        } => {
            msg!("Instruction: create exchange_rate");
            create_exchange_rate::process(program_id, accounts, voting_mint_bump, idx, er)
        }
        GovInstruction::CreateVoter {
            voter_bump,
            voter_weight_record_bump,
        } => {
            msg!("Instruction: create voter");
            create_voter::process(program_id, accounts, voter_bump, voter_weight_record_bump)
        }
        GovInstruction::CreateDeposit { kind, amount, days } => {
            msg!("Instruction: create deposit");
            create_deposit::process(program_id, accounts, kind, amount, days)
        }
    }
}
