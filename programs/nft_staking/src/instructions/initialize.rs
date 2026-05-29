use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use mpl_core::accounts::BaseCollectionV1;

use crate::{error::ErrorCode, Config, CONFIG, REWARDS_MINT, UPDATE_AUTH};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer=admin,
        space= Config::DISCRIMINATOR.len()+ Config::INIT_SPACE,
        seeds=[CONFIG, collection.key().as_ref()],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(has_one= update_authority @ ErrorCode::InvalidUpdateAuthority)]
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK:this account is not initialized and is being used for signing purposes only, we verify that derives from the correct seeds
    #[account(
        seeds=[UPDATE_AUTH,collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer=admin,
        mint::decimals=6,
        mint::authority=config,
        seeds=[REWARDS_MINT,collection.key().as_ref()],
        bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Initialize<'info> {
    pub fn init(
        &mut self,
        rewards_bps: u16,
        freeze_period: u16,
        bumps: InitializeBumps,
    ) -> Result<()> {
        self.config.set_inner(Config {
            rewards_bps,
            freeze_period,
            rewards_bump: bumps.rewards_mint,
            bump: bumps.config,
            staked_count: 0,
        });
        Ok(())
    }
}
