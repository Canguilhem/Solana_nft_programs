use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    error::ErrorCode, is_native_payment, refund_spl_from_vault, transfer_sol_from, Offer,
    NATIVE_PAYMENT_MINT, OFFER, OFFER_VAULT,
};

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validated via offer seeds
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        close = maker,
        seeds = [
            OFFER,
            asset.key().as_ref(),
            maker.key().as_ref(),
            NATIVE_PAYMENT_MINT.as_ref(),
        ],
        bump = offer.bump,
        has_one = maker,
        has_one = asset,
        constraint = is_native_payment(&offer.payment_mint) @ ErrorCode::InvalidPaymentMint,
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        seeds = [
            OFFER_VAULT,
            asset.key().as_ref(),
            maker.key().as_ref(),
            NATIVE_PAYMENT_MINT.as_ref(),
        ],
        bump = offer.vault_bump,
    )]
    pub offer_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CancelOffer<'info> {
    pub fn cancel_offer(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            OFFER_VAULT,
            self.offer.asset.as_ref(),
            self.offer.maker.as_ref(),
            NATIVE_PAYMENT_MINT.as_ref(),
            &[self.offer.vault_bump],
        ]];

        transfer_sol_from(
            &self.system_program.to_account_info(),
            &self.offer_vault.to_account_info(),
            &self.maker.to_account_info(),
            self.offer.price,
            Some(signer_seeds),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CancelOfferSpl<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validated via offer seeds
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        close = maker,
        seeds = [
            OFFER,
            asset.key().as_ref(),
            maker.key().as_ref(),
            payment_mint.key().as_ref(),
        ],
        bump = offer.bump,
        has_one = maker,
        has_one = asset,
        constraint = !is_native_payment(&offer.payment_mint) @ ErrorCode::InvalidPaymentMint,
        constraint = offer.payment_mint == payment_mint.key() @ ErrorCode::PaymentMintMismatch,
    )]
    pub offer: Account<'info, Offer>,

    /// CHECK: PDA authority for offer vault ATA
    #[account(
        seeds = [
            OFFER_VAULT,
            asset.key().as_ref(),
            maker.key().as_ref(),
            payment_mint.key().as_ref(),
        ],
        bump = offer.vault_bump,
    )]
    pub offer_vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = offer_vault_authority,
        associated_token::token_program = token_program,
    )]
    pub offer_vault_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_payment_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> CancelOfferSpl<'info> {
    pub fn cancel_offer_spl(&mut self) -> Result<()> {
        let mint_key = self.payment_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            OFFER_VAULT,
            self.offer.asset.as_ref(),
            self.offer.maker.as_ref(),
            mint_key.as_ref(),
            &[self.offer.vault_bump],
        ]];

        let decimals = crate::mint_decimals(&self.payment_mint)?;
        refund_spl_from_vault(
            &self.token_program.to_account_info(),
            &self.offer_vault_ata.to_account_info(),
            &self.payment_mint.to_account_info(),
            &self.maker_payment_ata.to_account_info(),
            &self.offer_vault_authority.to_account_info(),
            self.offer.price,
            decimals,
            signer_seeds,
        )
    }
}
