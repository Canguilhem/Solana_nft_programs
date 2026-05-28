use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{AddPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    programs::MPL_CORE_ID,
    types::{
        Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority,
    },
};

use crate::{error::ErrorCode, Config, CONFIG, UPDATE_AUTH};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds=[CONFIG,collection.key().as_ref()],
        bump= config.bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        has_one = owner @ErrorCode::InvalidOwner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()) @ ErrorCode::InvalidUpdateAuthority
    )]
    pub asset: Account<'info, BaseAssetV1>,
    #[account(
        mut,
        has_one = update_authority @ ErrorCode::InvalidUpdateAuthority
    )]
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: sign only account dont need to be initialized
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

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: StakeBumps) -> Result<()> {
        // fetch existing attributes if they exists
        let existing_attr: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attrs, _)| attrs);

        // prepare attributes list to add or update based on the existing attributes
        let mut attributes_list: Vec<Attribute> = Vec::new();

        // loop on attributes and save only the ones that are not the staking attributes
        // if we find the "staked" attribute already present we need to make sure asset is not already staked
        if let Some(attributes) = &existing_attr {
            for attr in &attributes.attribute_list {
                if attr.key == "staked" {
                    require!(attr.value == "false", ErrorCode::AssetAlreadyStaked)
                } else if attr.key != "staked_at" && attr.key != "last_claim" {
                    attributes_list.push(attr.clone())
                }
            }
        }

        // add the staking attributes
        attributes_list.push(Attribute {
            key: "staked".to_string(),
            value: "true".to_string(),
        });
        attributes_list.push(Attribute {
            key: "staked_at".to_string(),
            value: Clock::get()?.unix_timestamp.to_string(),
        });
        attributes_list.push(Attribute {
            key: "last_claim".to_string(),
            value: "0".to_string(),
        });

        // now that we have the complete list of attributes we either add the plugin or update the existing one
        // the attribute plugin is an authority-managed plugin so it needs to be signed by the update_auth

        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            UPDATE_AUTH,
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        match existing_attr {
            None => {
                AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.asset.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.owner.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes(Attributes {
                        attribute_list: attributes_list,
                    }))
                    .init_authority(PluginAuthority::UpdateAuthority)
                    .invoke_signed(signer_seeds)?;
            }
            Some(_) => {
                UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.asset.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.owner.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes(Attributes {
                        attribute_list: attributes_list,
                    }))
                    .invoke_signed(signer_seeds)?;
            }
        };

        // Freeze the asset
        AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
            .init_authority(PluginAuthority::UpdateAuthority)
            .invoke()?;

            self.config.staked_count = self
            .config
            .staked_count
            .checked_add(1)
            .ok_or(ErrorCode::InvalidStakedCount)?; 

        Ok(())
    }
}
