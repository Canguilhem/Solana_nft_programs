use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;

use crate::{
    NATIVE_PAYMENT_MINT, OFFER, OFFER_VAULT, Offer, error::ErrorCode, is_native_payment, transfer_sol_from
};

#[derive(Accounts)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validated via offer seeds
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    #[account(
        init,
        payer = maker,
        seeds = [
            OFFER,
            asset.key().as_ref(),
            maker.key().as_ref(),
            NATIVE_PAYMENT_MINT.as_ref(),
        ],
        bump,
        space = Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE,
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
        bump,
    )]
    pub offer_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> MakeOffer<'info> {
    pub fn make_offer(&mut self, price: u64, bumps: &MakeOfferBumps) -> Result<()> {
        require!(price > 0, ErrorCode::InvalidPrice);

        transfer_sol_from(
            &self.system_program.to_account_info(),
            &self.maker.to_account_info(),
            &self.offer_vault.to_account_info(),
            price,
            None,
        )?;

        self.offer.set_inner(Offer {
            maker: self.maker.key(),
            asset: self.asset.key(),
            price,
            payment_mint: NATIVE_PAYMENT_MINT,
            bump: bumps.offer,
            vault_bump: bumps.offer_vault,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MakeOfferSpl<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validated via offer seeds
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    pub payment_mint: InterfaceAccount<'info, anchor_spl::token_interface::Mint>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_payment_ata: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    /// CHECK: PDA authority for the offer vault ATA
    #[account(
        seeds = [
            OFFER_VAULT,
            asset.key().as_ref(),
            maker.key().as_ref(),
            payment_mint.key().as_ref(),
        ],
        bump,
    )]
    pub offer_vault_authority: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = payment_mint,
        associated_token::authority = offer_vault_authority,
        associated_token::token_program = token_program,
    )]
    pub offer_vault_ata: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    #[account(
        init,
        payer = maker,
        seeds = [
            OFFER,
            asset.key().as_ref(),
            maker.key().as_ref(),
            payment_mint.key().as_ref(),
        ],
        bump,
        space = Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE,
    )]
    pub offer: Account<'info, Offer>,

    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> MakeOfferSpl<'info> {
    pub fn make_offer_spl(&mut self, price: u64, bumps: &MakeOfferSplBumps) -> Result<()> {
        require!(price > 0, ErrorCode::InvalidPrice);
        require!(
            !is_native_payment(&self.payment_mint.key()),
            ErrorCode::InvalidPaymentMint
        );

        let decimals = crate::mint_decimals(&self.payment_mint)?;
        crate::transfer_spl_checked(
            &self.token_program.to_account_info(),
            &self.maker_payment_ata.to_account_info(),
            &self.payment_mint.to_account_info(),
            &self.offer_vault_ata.to_account_info(),
            &self.maker.to_account_info(),
            price,
            decimals,
            None,
        )?;

        self.offer.set_inner(Offer {
            maker: self.maker.key(),
            asset: self.asset.key(),
            price,
            payment_mint: self.payment_mint.key(),
            bump: bumps.offer,
            vault_bump: bumps.offer_vault_authority,
        });

        Ok(())
    }
}
