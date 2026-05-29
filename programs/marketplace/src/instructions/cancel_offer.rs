use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{Offer, OFFER, OFFER_VAULT};

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validated via offer seeds
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        close=maker,
        seeds=[OFFER, asset.key().as_ref(), maker.key().as_ref()],
        bump= offer.bump,
        has_one = maker,
        has_one = asset,
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        seeds=[OFFER_VAULT,asset.key().as_ref(), maker.key().as_ref()],
        bump= offer.vault_bump,
    )]
    pub offer_vault: SystemAccount<'info>,

    // /// CHECK: confirm program_id
    // #[account(address= MPL_CORE_ID)]
    // pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    // pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> CancelOffer<'info> {
    pub fn cancel_offer(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"offer_vault",
            self.offer.asset.as_ref(),
            self.offer.maker.as_ref(),
            &[self.offer.vault_bump],
        ]];

        // refund lamport
        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.offer_vault.to_account_info(),
                    to: self.maker.to_account_info(),
                },
                signer_seeds,
            ),
            self.offer.price,
        )?;

        Ok(())
    }
}
