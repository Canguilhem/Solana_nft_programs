use anchor_lang::prelude::*;

#[constant]
pub const MARKETPLACE: &[u8] = b"marketplace";

#[constant]
pub const REWARDS: &[u8] = b"REWARDS";

#[constant]
pub const TREASURY: &[u8] = b"treasury";

#[constant]
pub const LISTING: &[u8] = b"listing";

#[constant]
pub const OFFER: &[u8] = b"offer";

#[constant]
pub const OFFER_VAULT: &[u8] = b"offer_vault";

/// Sentinel mint for native SOL listings/offers (32 zero bytes).
pub const NATIVE_PAYMENT_MINT: Pubkey = Pubkey::new_from_array([0u8; 32]);

pub fn is_native_payment(mint: &Pubkey) -> bool {
    *mint == NATIVE_PAYMENT_MINT
}
