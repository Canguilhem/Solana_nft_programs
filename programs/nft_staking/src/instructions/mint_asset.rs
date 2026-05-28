use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseCollectionV1, instructions::CreateV2CpiBuilder, programs::MPL_CORE_ID,
};

use crate::UPDATE_AUTH;

#[derive(Accounts)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub asset: Signer<'info>,
    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: This account is not initialized and is being used for signing purposes only, we verify that derieves form the correct seeds
    #[account(
        seeds=[UPDATE_AUTH, collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the MPL Core Program
    #[account(address= MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> MintAsset<'info> {
    pub fn mint_asset(&mut self, name: String, uri: String, bumps: MintAssetBumps) -> Result<()> {
        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            UPDATE_AUTH,
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
