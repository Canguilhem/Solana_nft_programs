use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::ErrorCode, Offer, OFFER, OFFER_VAULT};

#[derive(Accounts)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validated via offer seeds
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    #[account(
        init,
        payer=maker,
        seeds=[OFFER, asset.key().as_ref(), maker.key().as_ref()],
        bump,
        space= Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        seeds=[OFFER_VAULT,asset.key().as_ref(), maker.key().as_ref()],
        bump,
    )]
    pub offer_vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> MakeOffer<'info> {
    pub fn make_offer(&mut self, price: u64, bumps: &MakeOfferBumps) -> Result<()> {
        require!(price > 0, ErrorCode::InvalidPrice);

        // transfer lamport
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.maker.to_account_info(),
                    to: self.offer_vault.to_account_info(),
                },
            ),
            price,
        )?;

        //  register the offer
        self.offer.set_inner(Offer {
            maker: self.maker.key(),
            asset: self.asset.key(),
            price,
            bump: bumps.offer,
            vault_bump: bumps.offer_vault,
        });

        Ok(())
    }
}
