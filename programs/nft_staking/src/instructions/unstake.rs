use anchor_lang::{prelude::*, solana_program::clock::Clock};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::MintToChecked,
    token_interface::{mint_to_checked, Mint, TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::UpdatePluginV1CpiBuilder,
    programs::MPL_CORE_ID,
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority},
};

use crate::{error::ErrorCode, Config, CONFIG, REWARDS_MINT, UPDATE_AUTH};

const SECONDS_PER_DAY: i64 = 86400;

#[derive(Accounts)]
pub struct Unstake<'info> {
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

    #[account(
        mut,
        seeds=[REWARDS_MINT, collection.key().as_ref()],
        bump= config.rewards_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer= owner,
        associated_token::mint= rewards_mint,
        associated_token::authority = owner,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the MPL Core Program
    #[account(address= MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self, bumps: UnstakeBumps) -> Result<()> {
        // fetch existing attributes if they exists
        let existing_attr: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attrs, _)| attrs);

        // some attributes should exist
        require!(existing_attr.is_some(), ErrorCode::AssetNotStaked);

        let existing = existing_attr.unwrap();

        // prepare attributes list to add or update based on the existing attributes
        let mut attributes_list: Vec<Attribute> = Vec::with_capacity(existing.attribute_list.len());

        // additional auxiliary variables
        let now = Clock::get()?.unix_timestamp;
        let mut staked_timestamp: i64 = 0;
        let mut staked_time: i64;
        let mut last_claim: i64 = 0;

        for attribute in &existing.attribute_list {
            if attribute.key == "staked" {
                require!(attribute.value == "true", ErrorCode::AssetNotStaked);
                // pushing to attributes with updated values
                attributes_list.push(Attribute {
                    key: "staked".to_string(),
                    value: "false".to_string(),
                });
            } else if attribute.key == "staked_at" {
                staked_timestamp = staked_timestamp
                    .checked_add(
                        attribute
                            .value
                            .parse::<i64>()
                            .map_err(|_| ErrorCode::InvalidTimestamp)?,
                    )
                    .ok_or(ErrorCode::InvalidTimestamp)?;
                // compute time elapsed since staked (in seconds)
                staked_time = now
                    .checked_sub(staked_timestamp)
                    .ok_or(ErrorCode::InvalidTimestamp)?;
                // convert staked time in days
                staked_time = staked_time
                    .checked_div(SECONDS_PER_DAY)
                    .ok_or(ErrorCode::InvalidTimestamp)?;
                require!(
                    staked_time >= self.config.freeze_period as i64,
                    ErrorCode::FreezePeriodNotElapsed
                );

                // pushing to attributes with updated values
                attributes_list.push(Attribute {
                    key: "staked_at".to_string(),
                    value: "0".to_string(),
                });
            } else if attribute.key == "last_claim" {
                last_claim = attribute
                    .value
                    .parse::<i64>()
                    .map_err(|_| ErrorCode::InvalidTimestamp)?;
                // pushing to attributes with updated values
                attributes_list.push(Attribute {
                    key: "last_claim".to_string(),
                    value: "0".to_string(),
                });
            } else {
                attributes_list.push(attribute.clone());
            }
        }

        // prepare update_auth signer seeds
        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            UPDATE_AUTH,
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        // update asset attributes plugin (existing + updated values)
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

        // UnFreeze the asset
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(signer_seeds)?;

        // mint any remaining rewards to user

        let anchor = if last_claim == 0 {
            staked_timestamp
        } else {
            last_claim
        };
        let remaining_days = now
            .checked_sub(anchor)
            .ok_or(ErrorCode::InvalidTimestamp)?
            .checked_div(SECONDS_PER_DAY)
            .ok_or(ErrorCode::InvalidTimestamp)?;

        if remaining_days > 0 && self.config.rewards_bps != 0 {
            // amount =  staked_time * bps * (10^decimals) / 10_000
            let amount: u64 = (remaining_days as u64)
                .checked_mul(self.config.rewards_bps as u64)
                .ok_or(ErrorCode::InvalidRewardsBps)?
                .checked_mul(10u64.pow(self.rewards_mint.decimals as u32))
                .ok_or(ErrorCode::InvalidRewardsBps)?
                .checked_div(10000u64)
                .ok_or(ErrorCode::InvalidRewardsBps)?;

            // prepare config signer
            let config_seeds: &[&[&[u8]]] =
                &[&[CONFIG, collection_key.as_ref(), &[self.config.bump]]];

            mint_to_checked(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    MintToChecked {
                        mint: self.rewards_mint.to_account_info(),
                        to: self.user_rewards_ata.to_account_info(),
                        authority: self.config.to_account_info(),
                    },
                    config_seeds,
                ),
                amount,
                self.rewards_mint.decimals,
            )?;
        }

        self.config.staked_count = self
            .config
            .staked_count
            .checked_sub(1)
            .ok_or(ErrorCode::InvalidStakedCount)?;
        Ok(())
    }
}
