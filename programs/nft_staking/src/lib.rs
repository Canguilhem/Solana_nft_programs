// #![allow(unexpected_cfgs,deprecated,ambiguous_glob_imports)]
#![allow(deprecated)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5MmdPvTaEoLaufFuun6a41y2o6ax5d1XkJd1CNg4Evye");

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        rewards_bps: u16,
        freeze_period: u16,
    ) -> Result<()> {
        ctx.accounts.init(rewards_bps, freeze_period, ctx.bumps)
    }

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.create_collection(name, uri, ctx.bumps)
    }

    pub fn mint_asset(ctx: Context<MintAsset>, name: String, uri: String) -> Result<()> {
        ctx.accounts.mint_asset(name, uri, ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(ctx.bumps)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(ctx.bumps)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        ctx.accounts.claim(ctx.bumps)
    }
}
