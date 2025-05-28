use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

pub use instructions::init_pool::*;

declare_id!("D84sXEooUu4FJ2EqdULee5HoELxb6Nfs1hiiLS6N73yp");

#[program]
pub mod liquidity_pool_project {
    use super::*;

    pub fn initialize_liquidity_pool(ctx: Context<InitializeLiquidityPool>, fees: u8) -> Result<()> {
        ctx.accounts.init_liquidit_pool(fees,&ctx.bumps)?;
        Ok(())
    }
}

