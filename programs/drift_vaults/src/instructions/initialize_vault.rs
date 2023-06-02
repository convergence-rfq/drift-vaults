use crate::{error::ErrorCode, Size, Vault};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use drift::cpi::accounts::{InitializeUser, InitializeUserStats};
use drift::program::Drift;
use drift::state::spot_market::SpotMarket;

pub fn initialize_vault(
    ctx: Context<InitializeVault>,
    name: [u8; 32],
    spot_market_index: u16,
) -> Result<()> {
    let bump = ctx.bumps.get("vault").ok_or(ErrorCode::Default)?;

    let signature_seeds = Vault::get_vault_signer_seeds(name.as_ref(), bump);
    let signers = &[&signature_seeds[..]];
    let cpi_program = ctx.accounts.drift_program.to_account_info().clone();
    let cpi_accounts = InitializeUserStats {
        user_stats: ctx.accounts.drift_user_stats.clone(),
        state: ctx.accounts.drift_state.clone(),
        authority: ctx.accounts.vault.to_account_info().clone(),
        payer: ctx.accounts.payer.to_account_info().clone(),
        rent: ctx.accounts.rent.to_account_info().clone(),
        system_program: ctx.accounts.system_program.to_account_info().clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    drift::cpi::initialize_user_stats(cpi_ctx)?;

    let cpi_program = ctx.accounts.drift_program.to_account_info().clone();
    let cpi_accounts = InitializeUser {
        user_stats: ctx.accounts.drift_user_stats.clone(),
        user: ctx.accounts.drift_user.clone(),
        state: ctx.accounts.drift_state.clone(),
        authority: ctx.accounts.vault.to_account_info().clone(),
        payer: ctx.accounts.payer.to_account_info().clone(),
        rent: ctx.accounts.rent.to_account_info().clone(),
        system_program: ctx.accounts.system_program.to_account_info().clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    let sub_account_id = 0_u16;
    drift::cpi::initialize_user(cpi_ctx, sub_account_id, name)?;

    let mut vault = ctx.accounts.vault.load_init()?;
    vault.name = name;
    vault.pubkey = *ctx.accounts.vault.to_account_info().key;
    vault.authority = *ctx.accounts.authority.key;
    vault.user_stats = *ctx.accounts.drift_user_stats.key;
    vault.user = *ctx.accounts.drift_user.key;
    vault.token_account = *ctx.accounts.token_account.to_account_info().key;
    vault.spot_market_index = spot_market_index;
    vault.bump = *bump;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    name: [u8; 32],
    spot_market_index: u16,
)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        seeds = [b"vault", name.as_ref()],
        space = Vault::SIZE,
        bump,
        payer = payer
    )]
    pub vault: AccountLoader<'info, Vault>,
    #[account(
        init,
        seeds = [b"vault_token_account".as_ref(), vault.key().as_ref()],
        bump,
        payer = payer,
        token::mint = drift_spot_market_mint,
        token::authority = vault
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: checked in drift cpi
    #[account(mut)]
    pub drift_user_stats: AccountInfo<'info>,
    /// CHECK: checked in drift cpi
    #[account(mut)]
    pub drift_user: AccountInfo<'info>,
    /// CHECK: checked in drift cpi
    #[account(mut)]
    pub drift_state: AccountInfo<'info>,
    #[account(
        constraint = drift_spot_market.load()?.market_index == spot_market_index
    )]
    pub drift_spot_market: AccountLoader<'info, SpotMarket>,
    #[account(
        constraint = drift_spot_market.load()?.mint.eq(&drift_spot_market_mint.key())
    )]
    pub drift_spot_market_mint: Box<Account<'info, Mint>>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub drift_program: Program<'info, Drift>,
    pub token_program: Program<'info, Token>,
}
