use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

declare_id!("5QkVbdLuVpbchKgFUtRrZoRiCyoqh5ANsnZuGZ9bjpzq");

#[program]
pub mod yien {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, params: InitParams) -> Result<()> {
        let config = &mut ctx.accounts.protocol_config;
        config.min_collateral_ratio = params.min_collateral_ratio;
        config.liquidation_penalty = params.liquidation_penalty;
        config.open_fee = params.open_fee;
        config.treasury = ctx.accounts.treasury.key();
        config.stability_pool = ctx.accounts.stability_pool.key();
        config.authority = ctx.accounts.authority.key();
        config.bump = ctx.bumps.protocol_config;

        Ok(())
    }

    pub fn open_position(ctx: Context<OpenPosition>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.protocol_config;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_collateral_account.to_account_info(),
            to: ctx.accounts.vault_collateral_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        let fee_amount = (amount * config.open_fee as u64) / 10000;
        if fee_amount > 0 {
            let fee_cpi_accounts = Transfer {
                from: ctx.accounts.user_collateral_account.to_account_info(),
                to: ctx.accounts.treasury_collateral_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };
            let fee_cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                fee_cpi_accounts,
            );
            transfer(fee_cpi_ctx, fee_amount)?;
        }

        let position = &mut ctx.accounts.farm_position;
        position.owner = ctx.accounts.user.key();
        position.collateral_mint = ctx.accounts.collateral_mint.key();
        position.collateral_amount = amount - fee_amount;
        position.bump = ctx.bumps.farm_position;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + ProtocolConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + StabilityPool::INIT_SPACE,
        seeds = [b"stability_pool"],
        bump
    )]
    pub stability_pool: Account<'info, StabilityPool>,

    /// CHECK: Treasury wallet to collect fees
    pub treasury: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + FarmPosition::INIT_SPACE,
        seeds = [b"position", user.key().as_ref(), collateral_mint.key().as_ref()],
        bump
    )]
    pub farm_position: Account<'info, FarmPosition>,
    pub collateral_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = user_collateral_account.owner == user.key(),
        constraint = user_collateral_account.mint == collateral_mint.key()
    )]
    pub user_collateral_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"vault", collateral_mint.key().as_ref()],
        bump
    )]
    pub vault_collateral_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"treasury_vault", collateral_mint.key().as_ref()],
        bump
    )]
    pub treasury_collateral_account: Account<'info, TokenAccount>,

    pub protocol_config: Account<'info, ProtocolConfig>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct ProtocolConfig {
    pub min_collateral_ratio: u64,
    pub liquidation_penalty: u64,
    pub open_fee: u64,
    pub treasury: Pubkey,
    pub stability_pool: Pubkey,
    pub authority: Pubkey,
    pub bump: u8,
}

impl ProtocolConfig {
    pub const INIT_SPACE: usize = 8 + 8 + 8 + 8 + 32 + 32 + 32 + 1;
}

#[account]
pub struct StabilityPool {
    pub total_deposited: u64,
    pub bump: u8,
}

#[account]
pub struct FarmPosition {
    pub owner: Pubkey,
    pub collateral_mint: Pubkey,
    pub collateral_amount: u64,
    pub bump: u8,
}

impl FarmPosition {
    pub const INIT_SPACE: usize = 32 + 32 + 8 + 1;
}

impl StabilityPool {
    pub const INIT_SPACE: usize = 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitParams {
    pub min_collateral_ratio: u64,
    pub liquidation_penalty: u64,
    pub open_fee: u64,
}
