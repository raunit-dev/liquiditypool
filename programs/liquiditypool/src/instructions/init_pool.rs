pub use crate::state::pool_config::LiquidityPoolConfig;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};


#[derive(Accounts)]
pub struct InitializeLiquidityPool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = creator,
        seeds = [b"poolconfig", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump,
        space = 8 + LiquidityPoolConfig::INIT_SPACE
    )]
    pub pool_config_account: Account<'info, LiquidityPoolConfig>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool_config_account,
        associated_token::token_program = token_program,
    )]
    pub vault_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = pool_config_account,
        associated_token::token_program = token_program,
    )]
    pub vault_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp_mint"],
        bump
    )]
    pub lp_mint_auth: SystemAccount<'info>,

    #[account(
        seeds = [b"pool_authority", pool_config_account.key().as_ref()],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl <'info> InitializeLiquidityPool <'info> {
    pub fn init_liquidit_pool(
        &mut self,
        fees : u8,
        bumps : &InitializeLiquidityPoolBumps,
    ) -> Result<()> {
        let pool_config_account = &mut self.pool_config_account;
        pool_config_account.set_inner(LiquidityPoolConfig {
            creator: self.creator.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            lp_mint: self.lp_mint.key(),
            vault_token_a: self.vault_token_a.key(),
            vault_token_b: self.vault_token_b.key(),
            pool_config_bump: bumps.pool_config_account,
            token_a_deposits: 0,
            token_b_deposits: 0,
            total_pool_value: 0,
            fees,
            created_at: Clock::get()?.unix_timestamp,
            is_active: true,
            authority: self.pool_authority.key(),
            lp_mint_auth: self.lp_mint_auth.key()
        });

        msg!("Pool config account: {:?}", pool_config_account);

        Ok(())
    }
}