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

pub fn list_spl_ix(
    ctx: &AnchorContext,
    maker: Pubkey,
    asset: Pubkey,
    collection: Option<Pubkey>,
    listing: Pubkey,
    payment_mint: Pubkey,
    price: u64,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::ListSpl {
            maker,
            asset,
            collection,
            payment_mint,
            listing,
            mpl_core_program: MPL_CORE_ID,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::ListSpl { price })
        .instruction()
        .unwrap()
}

pub fn buy_spl_ix(
    ctx: &mut AnchorContext,
    maker: Pubkey,
    taker: &Keypair,
    asset: Pubkey,
    collection: Option<Pubkey>,
    listing: Pubkey,
    marketplace: Pubkey,
    payment_mint: Pubkey,
    taker_payment_ata: Pubkey,
    treasury_authority: Pubkey,
    treasury_ata: Pubkey,
    rewards_mint: Pubkey,
    taker_rewards_ata: Pubkey,
) -> Instruction {
    let maker_payment_ata =
        associated_token::get_associated_token_address(&maker, &payment_mint);

    ctx.program()
        .accounts(marketplace::accounts::BuySpl {
            taker: taker.pubkey(),
            maker,
            asset,
            collection,
            marketplace,
            listing,
            payment_mint,
            taker_payment_ata,
            maker_payment_ata,
            treasury_authority,
            treasury_ata,
            rewards_mint,
            taker_rewards_ata,
            mpl_core_program: MPL_CORE_ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::BuySpl {})
        .instruction()
        .unwrap()
}

pub fn make_offer_spl_ix(
    ctx: &AnchorContext,
    maker: Pubkey,
    asset: Pubkey,
    payment_mint: Pubkey,
    maker_payment_ata: Pubkey,
    offer: Pubkey,
    offer_vault_authority: Pubkey,
    offer_vault_ata: Pubkey,
    price: u64,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::MakeOfferSpl {
            maker,
            asset,
            payment_mint,
            maker_payment_ata,
            offer_vault_authority,
            offer_vault_ata,
            offer,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::MakeOfferSpl { price })
        .instruction()
        .unwrap()
}

pub fn cancel_offer_spl_ix(
    ctx: &AnchorContext,
    maker: Pubkey,
    asset: Pubkey,
    payment_mint: Pubkey,
    offer: Pubkey,
    offer_vault_authority: Pubkey,
    offer_vault_ata: Pubkey,
    maker_payment_ata: Pubkey,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::CancelOfferSpl {
            maker,
            asset,
            payment_mint,
            offer,
            offer_vault_authority,
            offer_vault_ata,
            maker_payment_ata,
            token_program: token::ID,
        })
        .args(marketplace::instruction::CancelOfferSpl {})
        .instruction()
        .unwrap()
}

pub fn accept_offer_spl_ix(
    ctx: &AnchorContext,
    maker: Pubkey,
    taker: Pubkey,
    asset: Pubkey,
    collection: Option<Pubkey>,
    marketplace: Pubkey,
    payment_mint: Pubkey,
    rewards_mint: Pubkey,
    taker_rewards_ata: Pubkey,
    taker_payment_ata: Pubkey,
    treasury_authority: Pubkey,
    treasury_ata: Pubkey,
    offer: Pubkey,
    offer_vault_authority: Pubkey,
    offer_vault_ata: Pubkey,
) -> Instruction {
    let maker_payment_ata =
        associated_token::get_associated_token_address(&maker, &payment_mint);

    ctx.program()
        .accounts(marketplace::accounts::AcceptOfferSpl {
            taker,
            maker,
            asset,
            collection,
            marketplace,
            payment_mint,
            rewards_mint,
            taker_rewards_ata,
            taker_payment_ata,
            maker_payment_ata,
            treasury_authority,
            treasury_ata,
            offer,
            offer_vault_authority,
            offer_vault_ata,
            mpl_core_program: MPL_CORE_ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::AcceptOfferSpl {})
        .instruction()
        .unwrap()
}

pub fn withdraw_fee_spl_ix(
    ctx: &AnchorContext,
    admin: Pubkey,
    marketplace: Pubkey,
    payment_mint: Pubkey,
    treasury_authority: Pubkey,
    treasury_ata: Pubkey,
    admin_payment_ata: Pubkey,
) -> Instruction {
    ctx.program()
        .accounts(marketplace::accounts::WithdrawFeeSpl {
            admin,
            payment_mint,
            treasury_authority,
            treasury_ata,
            admin_payment_ata,
            marketplace,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
            token_program: token::ID,
        })
        .args(marketplace::instruction::WithdrawFeeSpl {})
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
