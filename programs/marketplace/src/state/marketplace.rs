use anchor_lang::prelude::*;

use crate::error::ErrorCode;

#[account]
#[derive(InitSpace)]
pub struct Marketplace {
    pub admin: Pubkey,
    pub fee: u16,
    pub bump: u8,
    pub treasury_bump: u8,
    pub rewards_bump: u8,
    #[max_len(32)]
    pub name: String,
}

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub maker: Pubkey,
    pub asset: Pubkey,
    pub price: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub maker: Pubkey,
    pub asset: Pubkey,
    pub price: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

pub fn split_price(price: u64, fee_bps: u16) -> Result<(u64, u64)> {
    let fee = (price as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(ErrorCode::MathError)?
        .checked_div(10_000)
        .ok_or(ErrorCode::MathError)? as u64;
    let net = price.checked_sub(fee).ok_or(ErrorCode::MathError)?;
    Ok((fee, net))
}
