use anchor_lang::solana_program::system_program;
use anchor_litesvm::{AnchorContext, Instruction, Keypair, Pubkey, Signer};
use anchor_spl::{associated_token, token};

use super::constants::MPL_CORE_ID;

pub fn initialize_ix(
    ctx: &AnchorContext,
    admin: Pubkey,
    marketplace: Pubkey,
    treasury: Pubkey,
    rewards_mint: Pubkey,
    name: String,
    fee: u16,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::Initialize {
            admin,
            marketplace,
            treasury,
            rewards_mint,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::Initialize { name, fee })
        .instruction()
        .unwrap()
}

pub fn list_ix(
    ctx: &AnchorContext,
    maker: Pubkey,
    asset: Pubkey,
    collection: Option<Pubkey>,
    listing: Pubkey,
    price: u64,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::List {
            maker,
            asset,
            collection,
            listing,
            mpl_core_program: MPL_CORE_ID,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::List { price })
        .instruction()
        .unwrap()
}

pub fn buy_ix(
    ctx: &mut AnchorContext,
    maker: Pubkey,
    taker: &Keypair,
    taker_rewards_ata: Pubkey,
    asset: Pubkey,
    collection: Option<Pubkey>,
    listing: Pubkey,
    marketplace: Pubkey,
    treasury: Pubkey,
    rewards_mint: Pubkey,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::Buy {
            maker,
            asset,
            collection,
            listing,
            mpl_core_program: MPL_CORE_ID,
            system_program: system_program::ID,
            token_program: token::ID,
            taker: taker.pubkey(),
            marketplace,
            treasury,
            rewards_mint,
            taker_rewards_ata,
            associated_token_program: associated_token::ID,
        })
        .args(marketplace::instruction::Buy {})
        .instruction()
        .unwrap()
}

pub fn delist_ix(
    ctx: &AnchorContext,
    maker: Pubkey,
    asset: Pubkey,
    collection: Option<Pubkey>,
    listing: Pubkey,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::Delist {
            maker,
            asset,
            collection,
            listing,
            mpl_core_program: MPL_CORE_ID,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::Delist {})
        .instruction()
        .unwrap()
}

pub fn make_offer(
    ctx: &AnchorContext,
    maker: Pubkey,
    asset: Pubkey,
    offer: Pubkey,
    offer_vault: Pubkey,
    price: u64,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::MakeOffer {
            maker,
            asset,
            offer,
            offer_vault,
            system_program: system_program::ID,
        })
        .args(marketplace::instruction::MakeOffer { price })
        .instruction()
        .unwrap()
}
pub fn cancel_offer(
    ctx: &AnchorContext,
    maker: Pubkey,
    asset: Pubkey,
    offer: Pubkey,
    offer_vault: Pubkey,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::CancelOffer {
            maker,
            asset,
            offer,
            offer_vault,
            system_program: system_program::ID,
        })
        .args(marketplace::instruction::CancelOffer {})
        .instruction()
        .unwrap()
}
pub fn accept_offer(
    ctx: &AnchorContext,
    maker: Pubkey,
    taker: Pubkey,
    asset: Pubkey,
    collection: Option<Pubkey>,
    marketplace: Pubkey,
    rewards_mint: Pubkey,
    taker_rewards_ata: Pubkey,
    treasury: Pubkey,
    offer: Pubkey,
    offer_vault: Pubkey,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::AcceptOffer {
            taker,
            maker,
            asset,
            collection,
            marketplace,
            rewards_mint,
            taker_rewards_ata,
            treasury,
            offer,
            offer_vault,
            mpl_core_program: MPL_CORE_ID,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        })
        .args(marketplace::instruction::AcceptOffer {})
        .instruction()
        .unwrap()
}
pub fn withdraw_fee(
    ctx: &AnchorContext,
    admin: Pubkey,

    marketplace: Pubkey,

    treasury: Pubkey,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::WithdrawFee {
            admin,
            treasury,
            marketplace,
            system_program: system_program::ID,
        })
        .args(marketplace::instruction::WithdrawFee {})
        .instruction()
        .unwrap()
}
