use anchor_lang::prelude::*;

declare_id!("D84sXEooUu4FJ2EqdULee5HoELxb6Nfs1hiiLS6N73yp");

#[program]
pub mod liquiditypool {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
