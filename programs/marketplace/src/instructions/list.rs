use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use mpl_core::{instructions::TransferV1CpiBuilder, programs::MPL_CORE_ID};


use crate::{Listing, Marketplace};

#[derive(Accounts)]
#[instruction(name:String)]
pub struct List<'info>{
    #[account(mut)]
    pub maker:Signer<'info>,
    
    /// CHECK: validated during cpi transfer by mpl-core  
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: validated during cpi transfer by mpl-core  
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    #[account(
        init,
        payer=maker,
        seeds=[b"listing", asset.key().as_ref()],
        bump,
        space= Listing::DISCRIMINATOR.len() + Listing::INIT_SPACE
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: confirm program_id
    #[account(address= MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info,System>,
    pub token_program: Interface<'info,TokenInterface>
}

impl <'info>List<'info>{
    pub fn list_asset(&mut self,price:u64, bumps:&ListBumps)-> Result<()>{
        
        // register listing
        self.listing.set_inner(Listing { 
            maker: self.maker.key(), 
            asset: self.asset.key(), 
            price, 
            bump: bumps.listing 
        });

        // transfer ownership
        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|a| a.as_ref()))
            .payer(&self.maker.to_account_info())
            .authority(Some(&self.maker.to_account_info()))
            .new_owner(&self.listing.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke()?;

        Ok(())

        
    }
}