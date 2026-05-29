mod common;

use anchor_lang::solana_program::{msg, native_token::LAMPORTS_PER_SOL};
use anchor_litesvm::{AssertionHelpers, Pubkey, Signer, TestHelpers};
use anchor_spl::associated_token::get_associated_token_address;
use marketplace::{Listing, Offer, OFFER};

use common::{
    assert_nft_owner, buy_ix, delist_ix, get_pdas, init_marketplace, initialize_ix, list_ix,
    mint_test_nft, setup_marketplace_with_mpl_core, LISTING,
};

use crate::common::{
    constants::OFFER_VAULT,
    instructions::{accept_offer, cancel_offer, make_offer, withdraw_fee}, setup::expected_split,
};

#[test]
fn test_initialize_marketplace() {
    let mut ctx = setup_marketplace_with_mpl_core();
    let admin = ctx.payer().insecure_clone();

    let (marketplace, _treasury, _rewards_mint) = init_marketplace(&mut ctx);

    ctx.svm.assert_account_exists(&marketplace);

    let mp: marketplace::Marketplace = ctx.get_account(&marketplace).unwrap();
    assert_eq!(mp.fee, 300);
    assert_eq!(mp.name, "test");
    assert_eq!(mp.admin, admin.pubkey());
}

#[test]
fn test_listing() {
    let mut ctx = setup_marketplace_with_mpl_core();

    let alice = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();
    let nft = mint_test_nft(&mut ctx, &alice);

    let (listing, _) =
        Pubkey::find_program_address(&[LISTING, nft.asset.as_ref()], &marketplace::id());

    let ix = list_ix(
        &ctx,
        alice.pubkey(),
        nft.asset,
        Some(nft.collection),
        listing,
        1_000_000_000,
    );

    ctx.execute_instruction(ix, &[&alice])
        .unwrap()
        .assert_success();

    let list: Listing = ctx.get_account(&listing).unwrap();
    assert_nft_owner(&ctx, nft.asset, listing);

    assert_eq!(list.price, 1_000_000_000);
    assert_eq!(list.asset, nft.asset);
    assert_eq!(list.maker, alice.pubkey());
}

#[test]
fn test_buy() {
    let mut ctx = setup_marketplace_with_mpl_core();
    let (marketplace, treasury, rewards_mint) = init_marketplace(&mut ctx);

    let alice = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();
    let bob = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();

    let nft = mint_test_nft(&mut ctx, &alice);

    let (listing, _) =
        Pubkey::find_program_address(&[LISTING, nft.asset.as_ref()], &marketplace::id());

    let ix = list_ix(
        &ctx,
        alice.pubkey(),
        nft.asset,
        Some(nft.collection),
        listing,
        1_000_000_000,
    );

    let tx_res = ctx.execute_instruction(ix, &[&alice]).unwrap();
    tx_res.assert_success();

    assert_nft_owner(&ctx, nft.asset, listing);

    let taker_rewards_ata = get_associated_token_address(&bob.pubkey(), &rewards_mint);
    let bob_balance_before = ctx.svm.get_balance(&bob.pubkey()).unwrap();
    let alice_balance_before = ctx.svm.get_balance(&alice.pubkey()).unwrap();

    let buy_ix = buy_ix(
        &mut ctx,
        alice.pubkey(),
        &bob,
        taker_rewards_ata,
        nft.asset,
        Some(nft.collection),
        listing,
        marketplace,
        treasury,
        rewards_mint,
    );

    ctx.execute_instruction(buy_ix, &[&bob])
        .unwrap()
        .assert_success();

    let bob_balance_after = ctx.svm.get_balance(&bob.pubkey()).unwrap();
    let alice_balance_after = ctx.svm.get_balance(&alice.pubkey()).unwrap();
    msg!(
        "bob_balance before {} after {}",
        bob_balance_before.to_string(),
        bob_balance_after.to_string()
    );
    msg!(
        "alice_balance before {} after {}",
        alice_balance_before.to_string(),
        alice_balance_after.to_string()
    );
    assert_nft_owner(&ctx, nft.asset, bob.pubkey());
}

#[test]
fn cancel_listing() {
    let mut ctx = setup_marketplace_with_mpl_core();
    let (_marketplace, _treasury, _rewards_mint) = init_marketplace(&mut ctx);

    let alice = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();

    let nft = mint_test_nft(&mut ctx, &alice);

    let (listing, _) =
        Pubkey::find_program_address(&[LISTING, nft.asset.as_ref()], &marketplace::id());

    let ix = list_ix(
        &ctx,
        alice.pubkey(),
        nft.asset,
        Some(nft.collection),
        listing,
        1_000_000_000,
    );

    let tx_res = ctx.execute_instruction(ix, &[&alice]).unwrap();
    tx_res.assert_success();

    assert_nft_owner(&ctx, nft.asset, listing);

    let delist_ix = delist_ix(
        &ctx,
        alice.pubkey(),
        nft.asset,
        Some(nft.collection),
        listing,
    );
    let tx_res = ctx.execute_instruction(delist_ix, &[&alice]).unwrap();
    tx_res.assert_success();

    ctx.svm.assert_account_closed(&listing);
    assert_nft_owner(&ctx, nft.asset, alice.pubkey());
}

#[test]
fn test_make_offer() {
    let mut ctx = setup_marketplace_with_mpl_core();

    let alice = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();
    let nft = mint_test_nft(&mut ctx, &alice);

    let bob = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();

    let bob_balance_before = ctx.svm.get_balance(&bob.pubkey()).unwrap();

    let (offer, _) = Pubkey::find_program_address(
        &[OFFER, nft.asset.as_ref(), bob.pubkey().as_ref()],
        &marketplace::id(),
    );

    let (offer_vault, _) = Pubkey::find_program_address(
        &[OFFER_VAULT, nft.asset.as_ref(), bob.pubkey().as_ref()],
        &marketplace::id(),
    );

    let ix = make_offer(
        &ctx,
        bob.pubkey(),
        nft.asset,
        offer,
        offer_vault,
        1_000_000_000,
    );

    ctx.execute_instruction(ix, &[&bob])
        .unwrap()
        .assert_success();

    let offer: Offer = ctx.get_account(&offer).unwrap();
    assert_eq!(offer.price, 1_000_000_000);
    assert_eq!(offer.asset, nft.asset);
    assert_eq!(offer.maker, bob.pubkey());
    // balance
    let bob_balance_after = ctx.svm.get_balance(&bob.pubkey()).unwrap();
    let vault_after = ctx.svm.get_balance(&offer_vault).unwrap();

    assert!(bob_balance_after < bob_balance_before - 1_000_000_000);
    assert_eq!(vault_after, 1_000_000_000)
}

#[test]
fn test_cancel_offer() {
    let mut ctx = setup_marketplace_with_mpl_core();

    let alice = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();
    let nft = mint_test_nft(&mut ctx, &alice);

    let bob = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();

    let bob_balance_before = ctx.svm.get_balance(&bob.pubkey()).unwrap();

    let (offer, _) = Pubkey::find_program_address(
        &[OFFER, nft.asset.as_ref(), bob.pubkey().as_ref()],
        &marketplace::id(),
    );

    let (offer_vault, _) = Pubkey::find_program_address(
        &[OFFER_VAULT, nft.asset.as_ref(), bob.pubkey().as_ref()],
        &marketplace::id(),
    );

    let ix = make_offer(
        &ctx,
        bob.pubkey(),
        nft.asset,
        offer,
        offer_vault,
        1_000_000_000,
    );

    ctx.execute_instruction(ix, &[&bob])
        .unwrap()
        .assert_success();

    let offer_acc: Offer = ctx.get_account(&offer).unwrap();
    assert_eq!(offer_acc.price, 1_000_000_000);
    assert_eq!(offer_acc.asset, nft.asset);
    assert_eq!(offer_acc.maker, bob.pubkey());
    // balance
    let bob_balance_after = ctx.svm.get_balance(&bob.pubkey()).unwrap();

    assert!(bob_balance_after < bob_balance_before - 1_000_000_000);

    // bob cancel the offer
    let ix = cancel_offer(&ctx, bob.pubkey(), nft.asset, offer, offer_vault);

    ctx.execute_instruction(ix, &[&bob])
        .unwrap()
        .assert_success();

    ctx.svm.assert_account_closed(&offer);
    let bob_after_cancel = ctx.svm.get_balance(&bob.pubkey()).unwrap();
    assert_eq!(ctx.svm.get_balance(&offer_vault).unwrap_or(0), 0);
    assert!(bob_after_cancel > bob_balance_after);
}

#[test]
fn test_accept_offer() {
    let mut ctx = setup_marketplace_with_mpl_core();
    let (marketplace, treasury, rewards_mint) = init_marketplace(&mut ctx);

    let alice = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();
    let nft = mint_test_nft(&mut ctx, &alice);

    let bob = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();

    let bob_balance_before = ctx.svm.get_balance(&bob.pubkey()).unwrap();

    let (offer, _) = Pubkey::find_program_address(
        &[OFFER, nft.asset.as_ref(), bob.pubkey().as_ref()],
        &marketplace::id(),
    );

    let (offer_vault, _) = Pubkey::find_program_address(
        &[OFFER_VAULT, nft.asset.as_ref(), bob.pubkey().as_ref()],
        &marketplace::id(),
    );

    let ix = make_offer(
        &ctx,
        bob.pubkey(),
        nft.asset,
        offer,
        offer_vault,
        1_000_000_000,
    );

    ctx.execute_instruction(ix, &[&bob])
        .unwrap()
        .assert_success();

    let offer_acc: Offer = ctx.get_account(&offer).unwrap();
    assert_eq!(offer_acc.price, 1_000_000_000);
    assert_eq!(offer_acc.asset, nft.asset);
    assert_eq!(offer_acc.maker, bob.pubkey());
    // balance
    let bob_balance_after = ctx.svm.get_balance(&bob.pubkey()).unwrap();

    assert!(bob_balance_after < bob_balance_before - 1_000_000_000);

    let taker_rewards_ata = get_associated_token_address(&alice.pubkey(), &rewards_mint);

    let alice_before_accept = ctx.svm.get_balance(&alice.pubkey()).unwrap();
    // alice accept the offer by sending her nft
    let ix = accept_offer(
        &ctx,
        bob.pubkey(),
        alice.pubkey(),
        nft.asset,
        Some(nft.collection),
        marketplace,
        rewards_mint,
        taker_rewards_ata,
        treasury,
        offer,
        offer_vault,
    );

    ctx.execute_instruction(ix, &[&alice])
        .unwrap()
        .assert_success();

    // account close / vault 0
    ctx.svm.assert_account_closed(&offer);
    assert_eq!(ctx.svm.get_balance(&offer_vault).unwrap_or(0), 0);

    // bob received the nft
    assert_nft_owner(&ctx, nft.asset, bob.pubkey());

    // alice receive offer.price - fees
    let alice_after_accept = ctx.svm.get_balance(&alice.pubkey()).unwrap();

    assert!(alice_before_accept < alice_after_accept)
}


#[test]
fn test_withdraw_fee() {
    let mut ctx = setup_marketplace_with_mpl_core();
    let admin = ctx.payer().insecure_clone();
    let (marketplace, treasury, rewards_mint) = init_marketplace(&mut ctx);

    let alice = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();
    let bob = ctx
        .svm
        .create_funded_account(10 * LAMPORTS_PER_SOL)
        .unwrap();

    let nft = mint_test_nft(&mut ctx, &alice);

    let (listing, _) =
        Pubkey::find_program_address(&[LISTING, nft.asset.as_ref()], &marketplace::id());

    let ix = list_ix(
        &ctx,
        alice.pubkey(),
        nft.asset,
        Some(nft.collection),
        listing,
        1_000_000_000,
    );

    let tx_res = ctx.execute_instruction(ix, &[&alice]).unwrap();
    tx_res.assert_success();

    assert_nft_owner(&ctx, nft.asset, listing);

    let taker_rewards_ata = get_associated_token_address(&bob.pubkey(), &rewards_mint);
    let bob_balance_before = ctx.svm.get_balance(&bob.pubkey()).unwrap();
    let alice_balance_before = ctx.svm.get_balance(&alice.pubkey()).unwrap();

    let buy_ix = buy_ix(
        &mut ctx,
        alice.pubkey(),
        &bob,
        taker_rewards_ata,
        nft.asset,
        Some(nft.collection),
        listing,
        marketplace,
        treasury,
        rewards_mint,
    );

    ctx.execute_instruction(buy_ix, &[&bob])
        .unwrap()
        .assert_success();

    // let bob_balance_after = ctx.svm.get_balance(&bob.pubkey()).unwrap();
    // let alice_balance_after = ctx.svm.get_balance(&alice.pubkey()).unwrap();
    let (fee, net) = expected_split(1_000_000_000);
    assert_eq!(ctx.svm.get_balance(&treasury).unwrap(), fee);

    assert_nft_owner(&ctx, nft.asset, bob.pubkey());

    let admin_before = ctx.svm.get_balance(&admin.pubkey()).unwrap();
    
    let ix = withdraw_fee(
        &ctx,
        admin.pubkey(),
        marketplace,
        treasury
    );

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();

        let admin_after = ctx.svm.get_balance(&admin.pubkey()).unwrap();

        assert!(admin_after > admin_before)
}
