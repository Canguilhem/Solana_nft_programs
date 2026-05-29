use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, MintTo},
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use mpl_core::{instructions::TransferV1CpiBuilder, programs::MPL_CORE_ID};

use crate::{
    disburse_spl_payment, error::ErrorCode, is_native_payment, mint_decimals, Listing, Marketplace,
    LISTING, MARKETPLACE, REWARDS, TREASURY,
};

#[derive(Accounts)]
pub struct BuySpl<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK:
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
    pub marketplace: Box<Account<'info, Marketplace>>,

    #[account(
        mut,
        close = maker,
        seeds = [LISTING, asset.key().as_ref()],
        bump = listing.bump,
        has_one = maker,
        has_one = asset,
        constraint = !is_native_payment(&listing.payment_mint) @ ErrorCode::InvalidPaymentMint,
        constraint = listing.payment_mint == payment_mint.key() @ ErrorCode::PaymentMintMismatch,
    )]
    pub listing: Box<Account<'info, Listing>>,

    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_payment_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = payment_mint,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_payment_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: PDA authority for per-mint treasury ATA
    #[account(
        seeds = [TREASURY, marketplace.key().as_ref(), payment_mint.key().as_ref()],
        bump,
    )]
    pub treasury_authority: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = payment_mint,
        associated_token::authority = treasury_authority,
        associated_token::token_program = token_program,
    )]
    pub treasury_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [REWARDS, marketplace.key().as_ref()],
        bump = marketplace.rewards_bump,
    )]
    pub rewards_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_rewards_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: confirm program_id
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> BuySpl<'info> {
    pub fn send_tokens(&mut self) -> Result<()> {
        let decimals = mint_decimals(&self.payment_mint)?;
        disburse_spl_payment(
            &self.token_program.to_account_info(),
            &self.taker_payment_ata.to_account_info(),
            &self.payment_mint.to_account_info(),
            &self.maker_payment_ata.to_account_info(),
            &self.treasury_ata.to_account_info(),
            &self.taker.to_account_info(),
            self.listing.price,
            self.marketplace.fee,
            decimals,
        )
    }

    pub fn receive_nft(&mut self) -> Result<()> {
        let asset_key = self.asset.key();
        let signer_seeds: &[&[&[u8]]] = &[&[LISTING, asset_key.as_ref(), &[self.listing.bump]]];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|a| a.as_ref()))
            .payer(&self.taker.to_account_info())
            .authority(Some(&self.listing.to_account_info()))
            .new_owner(&self.taker.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke_signed(signer_seeds)?;

        Ok(())
    }

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
}
