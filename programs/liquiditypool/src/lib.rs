#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("D84sXEooUu4FJ2EqdULee5HoELxb6Nfs1hiiLS6N73yp");

#[program]
pub mod liquiditypool {
    use super::*;

    pub fn initialize(ctx: Context<InitializeLiquidityPool>,fees: u8) -> Result<()> {
        ctx.accounts.init_liquidit_pool(&ctx.bumps,fees)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
