use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

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

impl StabilityPool {
    pub const INIT_SPACE: usize = 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitParams {
    pub min_collateral_ratio: u64,
    pub liquidation_penalty: u64,
    pub open_fee: u64,
}
