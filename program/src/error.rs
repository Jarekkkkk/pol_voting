use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum GovError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Invalid Rate")]
    InvalidRate,
    #[error("Authority Mismatch")]
    AuthorityMismatch,
    #[error("Registrar Mismatch")]
    RegistrarMismatch,
}

impl From<GovError> for ProgramError {
    fn from(e: GovError) -> ProgramError {
        ProgramError::Custom(e as u32)
    }
}
