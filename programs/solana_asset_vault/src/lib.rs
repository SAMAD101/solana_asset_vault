use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("B3G43MVazHAwnxHpg5R1u2bCUkm7FnCaMgh2ikViL7aN");

#[program]
pub mod asset_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVaultState>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.token_account = ctx.accounts.vault_token_account.key();
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let depositor = ctx.accounts.depositor.key();

        // Update or create user balance
        let user_balance = vault.user_balances.entry(depositor).or_insert(0);
        *user_balance += amount;

        let transfer_instruction = Transfer {
            from: ctx.accounts.depositor_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        };

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                transfer_instruction,
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let withdrawer = ctx.accounts.withdrawer.key();

        // Check if user has sufficient balance
        let user_balance = vault
            .user_balances
            .get_mut(&withdrawer)
            .ok_or(ErrorCode::InsufficientBalance)?;
        if *user_balance < amount {
            return Err(ErrorCode::InsufficientBalance.into());
        }

        // Update user balance
        *user_balance -= amount;

        // Transfer tokens from vault to user
        let binding = ctx.accounts.mint.key();
        let seeds = &[b"vault".as_ref(), binding.as_ref(), &[vault.bump]];
        let signer = &[&seeds[..]];

        let transfer_instruction = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.withdrawer_token_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_instruction,
                signer,
            ),
            amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVaultState<'info> {
    #[account(
        init,
        payer = authority,
        space = VaultState::INIT_SPACE,
        seeds = [b"vault".as_ref(), mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,
    pub depositor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault".as_ref(), mint.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, VaultState>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = vault,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = withdrawer,
    )]
    pub withdrawer_token_account: Account<'info, TokenAccount>,
    pub withdrawer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct VaultState {
    pub authority: Pubkey,
    pub token_account: Pubkey,
    pub bump: u8,
    pub user_balances: std::collections::BTreeMap<Pubkey, u64>,
}

const DISCRIMINATOR_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const U8_LENGTH: usize = 1;
const USER_BALANCE_LENGTH: usize = 32 + 8;
const MAX_USERS: usize = 10;

impl Space for VaultState {
    const INIT_SPACE: usize = DISCRIMINATOR_LENGTH
        + PUBLIC_KEY_LENGTH // authority
        + PUBLIC_KEY_LENGTH // token_account
        + U8_LENGTH // bump
        + 4 // for BTreeMap length prefix (u32)
        + (USER_BALANCE_LENGTH * MAX_USERS); // user_balances
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient balance for withdrawal")]
    InsufficientBalance,
}
