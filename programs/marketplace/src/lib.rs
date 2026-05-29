// #![allow(unexpected_cfgs,deprecated,ambiguous_glob_imports)]
#![allow(deprecated, unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GUJJmt5VTpf119kmza9TjRqFW13WXCf5BnMYLMzQHzct");

#[program]
pub mod marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, fee: u16) -> Result<()> {
        ctx.accounts.init(name, fee, ctx.bumps)
    }

    /// List an NFT for native SOL.
    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.list_asset(price, &ctx.bumps)
    }

    /// List an NFT for an SPL / Token-2022 payment mint.
    pub fn list_spl(ctx: Context<ListSpl>, price: u64) -> Result<()> {
        ctx.accounts.list_asset_spl(price, &ctx.bumps)
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.delist_asset()
    }

    /// Buy a native-SOL listing.
    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        ctx.accounts.send_sol()?;
        ctx.accounts.receive_nft()?;
        ctx.accounts.receive_rewards()
    }

    /// Buy an SPL-listed NFT.
    pub fn buy_spl(ctx: Context<BuySpl>) -> Result<()> {
        ctx.accounts.send_tokens()?;
        ctx.accounts.receive_nft()?;
        ctx.accounts.receive_rewards()
    }

    /// Escrow native SOL for an offer.
    pub fn make_offer(ctx: Context<MakeOffer>, price: u64) -> Result<()> {
        ctx.accounts.make_offer(price, &ctx.bumps)
    }

    /// Escrow SPL tokens for an offer.
    pub fn make_offer_spl(ctx: Context<MakeOfferSpl>, price: u64) -> Result<()> {
        ctx.accounts.make_offer_spl(price, &ctx.bumps)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        ctx.accounts.cancel_offer()
    }

    pub fn cancel_offer_spl(ctx: Context<CancelOfferSpl>) -> Result<()> {
        ctx.accounts.cancel_offer_spl()
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        ctx.accounts.transfer_sol()?;
        ctx.accounts.transfer_nft()?;
        ctx.accounts.receive_rewards()
    }

    pub fn accept_offer_spl(ctx: Context<AcceptOfferSpl>) -> Result<()> {
        ctx.accounts.transfer_tokens()?;
        ctx.accounts.transfer_nft()?;
        ctx.accounts.receive_rewards()
    }

    pub fn withdraw_fee(ctx: Context<WithdrawFee>) -> Result<()> {
        ctx.accounts.withdraw()
    }

    pub fn withdraw_fee_spl(ctx: Context<WithdrawFeeSpl>) -> Result<()> {
        ctx.accounts.withdraw(ctx.bumps.treasury_authority)
    }
}
