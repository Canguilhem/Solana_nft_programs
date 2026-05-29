use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid fee amount")]
    InvalidFeeAmount,
    #[msg("MathError")]
    MathError,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Unauthorized")]
    Unauthorized,
}
