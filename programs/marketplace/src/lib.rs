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

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.list_asset(price, &ctx.bumps)
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.delist_asset()
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        ctx.accounts.send_sol()?;
        ctx.accounts.receive_nft()?;
        ctx.accounts.receive_rewards()
    }

    pub fn make_offer(ctx: Context<MakeOffer>, price: u64) -> Result<()> {
        ctx.accounts.make_offer(price, &ctx.bumps)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        ctx.accounts.cancel_offer()
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        ctx.accounts.transfer_sol()?;
        ctx.accounts.transfer_nft()?;
        ctx.accounts.receive_rewards()
    }

    pub fn withdraw_fee(ctx: Context<WithdrawFee>) -> Result<()> {
        ctx.accounts.withdraw()
    }
}
