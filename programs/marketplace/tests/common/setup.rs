use anchor_lang::solana_program::system_program;
use anchor_litesvm::{
    AnchorContext, AnchorLiteSVM, Keypair, ProgramTestExt, Pubkey, Signer, TestHelpers,
};
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::{CreateCollectionV2Builder, CreateV2Builder},
};

use super::constants::{MARKETPLACE, MPL_CORE_ID, REWARDS, TREASURY};
use super::instructions::initialize_ix;

pub fn assert_nft_owner(ctx: &AnchorContext, asset: Pubkey, expected_owner: Pubkey) {
    let asset_account: BaseAssetV1 = ctx.get_account(&asset).unwrap();
    assert_eq!(asset_account.owner, expected_owner, "NFT owner mismatch");
}

/// LiteSVM with marketplace + MPL Core programs loaded.
pub fn setup_marketplace_with_mpl_core() -> AnchorContext {
    let mut ctx = AnchorLiteSVM::new()
        .deploy_program(
            marketplace::id(),
            include_bytes!("../../../../target/deploy/marketplace.so"),
        )
        .build();

    ctx.deploy_program(MPL_CORE_ID, include_bytes!("../fixtures/mpl_core.so"));

    ctx
}

pub fn init_marketplace(ctx: &mut AnchorContext) -> (Pubkey, Pubkey, Pubkey) {
    let admin = ctx.payer().insecure_clone();

    let name = "test".to_string();
    let (marketplace, treasury, rewards_mint) = get_pdas(&name);

    let ix = initialize_ix(
        ctx,
        admin.pubkey(),
        marketplace,
        treasury,
        rewards_mint,
        name,
        300,
    );

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();

    (marketplace, treasury, rewards_mint)
}

pub fn get_pdas(market_name: &str) -> (Pubkey, Pubkey, Pubkey) {
    let (marketplace, _) =
        Pubkey::find_program_address(&[MARKETPLACE, market_name.as_bytes()], &marketplace::id());
    let (treasury, _) =
        Pubkey::find_program_address(&[TREASURY, marketplace.as_ref()], &marketplace::id());
    let (rewards_mint, _) =
        Pubkey::find_program_address(&[REWARDS, marketplace.as_ref()], &marketplace::id());

    (marketplace, treasury, rewards_mint)
}

pub struct MintedNft {
    pub collection: Pubkey,
    pub asset: Pubkey,
    pub owner: Pubkey,
}

pub fn create_collection(ctx: &mut AnchorContext, payer: &Keypair, collection: &Keypair) {
    let ix = CreateCollectionV2Builder::new()
        .collection(collection.pubkey())
        .payer(payer.pubkey())
        .update_authority(Some(payer.pubkey()))
        .system_program(system_program::ID)
        .name("Test Collection".into())
        .uri("https://example.com/collection.json".into())
        .instruction();
    ctx.execute_instruction(ix, &[payer, collection])
        .unwrap()
        .assert_success();
}

pub fn mint_asset_in_collection(
    ctx: &mut AnchorContext,
    payer: &Keypair,
    collection: Pubkey,
    asset: &Keypair,
    owner: Pubkey,
) {
    let ix = CreateV2Builder::new()
        .asset(asset.pubkey())
        .collection(Some(collection))
        .payer(payer.pubkey())
        .owner(Some(owner))
        .system_program(system_program::ID)
        .name("Test NFT".into())
        .uri("https://example.com/nft.json".into())
        .instruction();
    ctx.execute_instruction(ix, &[payer, asset])
        .unwrap()
        .assert_success();
}

pub fn mint_test_nft(ctx: &mut AnchorContext, owner: &Keypair) -> MintedNft {
    let collection = Keypair::new();
    let asset = Keypair::new();
    create_collection(ctx, owner, &collection);
    mint_asset_in_collection(ctx, owner, collection.pubkey(), &asset, owner.pubkey());
    MintedNft {
        collection: collection.pubkey(),
        asset: asset.pubkey(),
        owner: owner.pubkey(),
    }
}

pub const TEST_FEE_BPS: u16 = 300;
pub fn expected_split(price: u64) -> (u64, u64) {
    marketplace::split_price(price, TEST_FEE_BPS).unwrap()
}