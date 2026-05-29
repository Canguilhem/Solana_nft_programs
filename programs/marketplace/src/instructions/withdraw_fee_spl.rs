use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    error::ErrorCode, is_native_payment, mint_decimals, withdraw_spl_treasury, Marketplace,
    MARKETPLACE, TREASURY,
};

#[derive(Accounts)]
pub struct WithdrawFeeSpl<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub payment_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA authority for per-mint treasury ATA
    #[account(
        seeds = [TREASURY, marketplace.key().as_ref(), payment_mint.key().as_ref()],
        bump,
    )]
    pub treasury_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = treasury_authority,
        associated_token::token_program = token_program,
    )]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = payment_mint,
        associated_token::authority = admin,
        associated_token::token_program = token_program,
    )]
    pub admin_payment_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [MARKETPLACE, marketplace.name.as_bytes()],
        bump = marketplace.bump,
        has_one = admin @ ErrorCode::Unauthorized,
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> WithdrawFeeSpl<'info> {
    pub fn withdraw(&mut self, treasury_bump: u8) -> Result<()> {
        require!(
            !is_native_payment(&self.payment_mint.key()),
            ErrorCode::InvalidPaymentMint
        );

        let amount = self.treasury_ata.amount;
        require!(amount > 0, ErrorCode::InvalidPrice);

        let decimals = mint_decimals(&self.payment_mint)?;
        let market_key = self.marketplace.key();
        let mint_key = self.payment_mint.key();

        let signer_seeds: &[&[&[u8]]] = &[&[
            TREASURY,
            market_key.as_ref(),
            mint_key.as_ref(),
            &[treasury_bump],
        ]];

        withdraw_spl_treasury(
            &self.token_program.to_account_info(),
            &self.treasury_ata.to_account_info(),
            &self.payment_mint.to_account_info(),
            &self.admin_payment_ata.to_account_info(),
            &self.treasury_authority.to_account_info(),
            amount,
            decimals,
            signer_seeds,
        )
    }
}
