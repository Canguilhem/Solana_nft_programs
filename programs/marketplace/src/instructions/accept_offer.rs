use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, MintTo},
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use mpl_core::{instructions::TransferV1CpiBuilder, programs::MPL_CORE_ID};

use crate::{
    disburse_sol_from_vault, error::ErrorCode, is_native_payment, Marketplace, Offer, MARKETPLACE,
    NATIVE_PAYMENT_MINT, OFFER, OFFER_VAULT, REWARDS, TREASURY,
};

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK: used as key for offer seeds
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    /// CHECK: validated during cpi transfer by mpl-core
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: validated during cpi transfer by mpl-core
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    #[account(
        seeds = [MARKETPLACE, marketplace.name.as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [REWARDS, marketplace.key().as_ref()],
        bump = marketplace.rewards_bump,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_rewards_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [TREASURY, marketplace.key().as_ref()],
        bump = marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,

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

    /// CHECK: confirm program_id
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> AcceptOffer<'info> {
    pub fn receive_rewards(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            MARKETPLACE,
            self.marketplace.name.as_bytes(),
            &[self.marketplace.bump],
        ]];

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.rewards_mint.to_account_info(),
                    to: self.taker_rewards_ata.to_account_info(),
                    authority: self.marketplace.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        Ok(())
    }

    pub fn transfer_sol(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            OFFER_VAULT,
            self.offer.asset.as_ref(),
            self.offer.maker.as_ref(),
            NATIVE_PAYMENT_MINT.as_ref(),
            &[self.offer.vault_bump],
        ]];

        disburse_sol_from_vault(
            &self.system_program.to_account_info(),
            &self.offer_vault.to_account_info(),
            &self.taker.to_account_info(),
            &self.treasury.to_account_info(),
            self.offer.price,
            self.marketplace.fee,
            signer_seeds,
        )
    }

    pub fn transfer_nft(&mut self) -> Result<()> {
        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|a| a.as_ref()))
            .payer(&self.taker.to_account_info())
            .authority(Some(&self.taker.to_account_info()))
            .new_owner(&self.maker.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke()?;

        Ok(())
    }
}
