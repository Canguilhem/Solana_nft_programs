use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid asset owner")]
    InvalidOwner,
    #[msg("Invalid update authority")]
    InvalidUpdateAuthority,
    #[msg("Asset already staked")]
    AssetAlreadyStaked,
    #[msg("Asset not staked")]
    AssetNotStaked,
    #[msg("Invalid Timestamp")]
    InvalidTimestamp,
    #[msg("Freeze period not elapsed")]
    FreezePeriodNotElapsed,
    #[msg("Invalid rewards bps")]
    InvalidRewardsBps,
    #[msg("Invalid staked count")]
    InvalidStakedCount,
}
