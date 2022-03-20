use solana_program::decode_error::DecodeError;
use solana_program::program_error::ProgramError;

/// Errors that may be returned by the program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenitisError {
    // 0
}

impl From<TokenitisError> for ProgramError {
    fn from(e: TokenitisError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for TokenitisError {
    fn type_of() -> &'static str {
        "TokenitisError"
    }
}

pub const MAX_STRING_SIZE: u64 = 300;
