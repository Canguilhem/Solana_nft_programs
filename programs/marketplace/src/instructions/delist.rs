use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use mpl_core::{instructions::TransferV1CpiBuilder, programs::MPL_CORE_ID};

use crate::{Listing, LISTING};

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validated during cpi transfer by mpl-core  
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: validated during cpi transfer by mpl-core  
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    #[account(
        mut,
        close= maker,
        seeds=[LISTING, asset.key().as_ref()],
        bump,
        has_one= asset,
        has_one= maker
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: confirm program_id
    #[account(address= MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Delist<'info> {
    pub fn delist_asset(&mut self) -> Result<()> {
        let asset_key = self.asset.key();

        let signer_seeds: &[&[&[u8]]] = &[&[b"listing", asset_key.as_ref(), &[self.listing.bump]]];

        // transfer ownership
        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|a| a.as_ref()))
            .payer(&self.maker.to_account_info())
            .authority(Some(&self.listing.to_account_info()))
            .new_owner(&self.maker.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
