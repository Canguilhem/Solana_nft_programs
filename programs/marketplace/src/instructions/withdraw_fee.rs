use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::ErrorCode, Marketplace, MARKETPLACE, TREASURY};

// admin should be allowed to withdraw amount from trasury
// should we send to specific recipient ?
#[derive(Accounts)]
pub struct WithdrawFee<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds=[TREASURY,marketplace.key().as_ref()],
        bump= marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,

    #[account(
        seeds=[MARKETPLACE, marketplace.name.as_bytes()],
        bump= marketplace.bump,
        has_one= admin @ ErrorCode::Unauthorized
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawFee<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        let market_key = self.marketplace.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"treasury",
            market_key.as_ref(),
            &[self.marketplace.treasury_bump],
        ]];

        // exclude rent from withdraw
        let rent_exempt = Rent::get()?.minimum_balance(self.treasury.to_account_info().data_len());

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.treasury.to_account_info(),
                    to: self.admin.to_account_info(),
                },
                signer_seeds,
            ),
            self.treasury
                .lamports()
                .checked_sub(rent_exempt)
                .ok_or(ErrorCode::MathError)?,
        )?;

        Ok(())
    }
}
