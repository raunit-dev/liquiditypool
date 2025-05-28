#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

pub use instructions::init_pool::*;
pub use instructions::deposit::*;

declare_id!("D84sXEooUu4FJ2EqdULee5HoELxb6Nfs1hiiLS6N73yp");

#[program]
pub mod liquidity_pool_project {
    use super::*;

    pub fn initialize_liquidity_pool(ctx: Context<InitializeLiquidityPool>, fees: u8) -> Result<()> {
        ctx.accounts.init_liquidit_pool(fees,&ctx.bumps)?;
        Ok(())
    }

    pub fn deposit_liquidity_pool(ctx: Context<DepositLiquidity>,amount_a: u64,amount_b: u64,min_lp_tokens: u64,price_feed_id_a: String,price_feed_id_b: String) -> Result<()> {
        ctx.accounts.deposit_liquidity(amount_a,amount_b,min_lp_tokens,price_feed_id_a,price_feed_id_b)?;
        Ok(())
    }
}

