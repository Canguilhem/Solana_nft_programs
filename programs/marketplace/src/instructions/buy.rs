use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, MintTo},
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use mpl_core::{instructions::TransferV1CpiBuilder, programs::MPL_CORE_ID};

use crate::{split_price, Listing, Marketplace, LISTING, MARKETPLACE, REWARDS, TREASURY};

#[derive(Accounts)]
pub struct Buy<'info> {
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
        seeds=[MARKETPLACE, marketplace.name.as_bytes()],
        bump= marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        close= maker,
        seeds=[LISTING, asset.key().as_ref()],
        bump= listing.bump,
        has_one= maker,
        has_one= asset
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        mut,
        seeds=[TREASURY,marketplace.key().as_ref()],
        bump= marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,

    #[account(
        mut,
        seeds=[REWARDS, marketplace.key().as_ref()],
        bump= marketplace.rewards_bump,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer= taker,
        associated_token::mint= rewards_mint,
        associated_token::authority= taker,
        associated_token::token_program= token_program
    )]
    pub taker_rewards_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: confirm program_id
    #[account(address= MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Buy<'info> {
    pub fn send_sol(&mut self) -> Result<()> {
        // compute fees
        let (fee, net) = split_price(self.listing.price, self.marketplace.fee)?;

        // send amount -> maker
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.taker.to_account_info(),
                    to: self.maker.to_account_info(),
                },
            ),
            net,
        )?;

        // send fees -> treasury
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.taker.to_account_info(),
                    to: self.treasury.to_account_info(),
                },
            ),
            fee,
        )?;

        Ok(())
    }

    pub fn receive_nft(&mut self) -> Result<()> {
        let asset_key = self.asset.key();

        let signer_seeds: &[&[&[u8]]] = &[&[b"listing", asset_key.as_ref(), &[self.listing.bump]]];

        // transfer ownership
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
            b"marketplace",
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
