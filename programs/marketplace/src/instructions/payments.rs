use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::token_interface::{self, Mint, TransferChecked};

use crate::{error::ErrorCode, split_price};

pub fn transfer_sol_from<'info>(
    system_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi = Transfer {
        from: from.clone(),
        to: to.clone(),
    };
    match signer_seeds {
        Some(seeds) => transfer(
            CpiContext::new_with_signer(system_program.clone(), cpi, seeds),
            amount,
        ),
        None => transfer(CpiContext::new(system_program.clone(), cpi), amount),
    }
}

pub fn disburse_sol_payment<'info>(
    system_program: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    recipient: &AccountInfo<'info>,
    treasury: &AccountInfo<'info>,
    price: u64,
    fee_bps: u16,
) -> Result<()> {
    let (fee, net) = split_price(price, fee_bps)?;
    transfer_sol_from(system_program, payer, recipient, net, None)?;
    transfer_sol_from(system_program, payer, treasury, fee, None)
}

pub fn disburse_sol_from_vault<'info>(
    system_program: &AccountInfo<'info>,
    vault: &AccountInfo<'info>,
    recipient: &AccountInfo<'info>,
    treasury: &AccountInfo<'info>,
    price: u64,
    fee_bps: u16,
    vault_signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let (fee, net) = split_price(price, fee_bps)?;
    transfer_sol_from(
        system_program,
        vault,
        recipient,
        net,
        Some(vault_signer_seeds),
    )?;
    transfer_sol_from(
        system_program,
        vault,
        treasury,
        fee,
        Some(vault_signer_seeds),
    )
}

pub fn transfer_spl_checked<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amount: u64,
    decimals: u8,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi = TransferChecked {
        from: from.clone(),
        mint: mint.clone(),
        to: to.clone(),
        authority: authority.clone(),
    };
    match signer_seeds {
        Some(seeds) => token_interface::transfer_checked(
            CpiContext::new_with_signer(token_program.clone(), cpi, seeds),
            amount,
            decimals,
        ),
        None => token_interface::transfer_checked(
            CpiContext::new(token_program.clone(), cpi),
            amount,
            decimals,
        ),
    }
}

pub fn disburse_spl_payment<'info>(
    token_program: &AccountInfo<'info>,
    payer_ata: &AccountInfo<'info>,
    payment_mint: &AccountInfo<'info>,
    maker_ata: &AccountInfo<'info>,
    treasury_ata: &AccountInfo<'info>,
    payer_authority: &AccountInfo<'info>,
    price: u64,
    fee_bps: u16,
    decimals: u8,
) -> Result<()> {
    let (fee, net) = split_price(price, fee_bps)?;
    transfer_spl_checked(
        token_program,
        payer_ata,
        payment_mint,
        maker_ata,
        payer_authority,
        net,
        decimals,
        None,
    )?;
    transfer_spl_checked(
        token_program,
        payer_ata,
        payment_mint,
        treasury_ata,
        payer_authority,
        fee,
        decimals,
        None,
    )
}

pub fn disburse_spl_from_vault<'info>(
    token_program: &AccountInfo<'info>,
    vault_ata: &AccountInfo<'info>,
    payment_mint: &AccountInfo<'info>,
    recipient_ata: &AccountInfo<'info>,
    treasury_ata: &AccountInfo<'info>,
    vault_authority: &AccountInfo<'info>,
    price: u64,
    fee_bps: u16,
    decimals: u8,
    vault_signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let (fee, net) = split_price(price, fee_bps)?;
    transfer_spl_checked(
        token_program,
        vault_ata,
        payment_mint,
        recipient_ata,
        vault_authority,
        net,
        decimals,
        Some(vault_signer_seeds),
    )?;
    transfer_spl_checked(
        token_program,
        vault_ata,
        payment_mint,
        treasury_ata,
        vault_authority,
        fee,
        decimals,
        Some(vault_signer_seeds),
    )
}

pub fn refund_spl_from_vault<'info>(
    token_program: &AccountInfo<'info>,
    vault_ata: &AccountInfo<'info>,
    payment_mint: &AccountInfo<'info>,
    recipient_ata: &AccountInfo<'info>,
    vault_authority: &AccountInfo<'info>,
    amount: u64,
    decimals: u8,
    vault_signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    transfer_spl_checked(
        token_program,
        vault_ata,
        payment_mint,
        recipient_ata,
        vault_authority,
        amount,
        decimals,
        Some(vault_signer_seeds),
    )
}

pub fn withdraw_spl_treasury<'info>(
    token_program: &AccountInfo<'info>,
    treasury_ata: &AccountInfo<'info>,
    payment_mint: &AccountInfo<'info>,
    admin_ata: &AccountInfo<'info>,
    treasury_authority: &AccountInfo<'info>,
    amount: u64,
    decimals: u8,
    treasury_signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidPrice);
    transfer_spl_checked(
        token_program,
        treasury_ata,
        payment_mint,
        admin_ata,
        treasury_authority,
        amount,
        decimals,
        Some(treasury_signer_seeds),
    )
}

pub fn mint_decimals(mint: &InterfaceAccount<Mint>) -> Result<u8> {
    Ok(mint.decimals)
}
