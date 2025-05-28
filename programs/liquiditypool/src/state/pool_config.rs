use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct LiquidityPoolConfig {
    pub creator: Pubkey,
    pub authority: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub lp_mint: Pubkey,
    pub vault_token_a: Pubkey,
    pub vault_token_b: Pubkey,
    pub lp_mint_auth: Pubkey,
    pub token_a_deposits: u64,
    pub token_b_deposits: u64,
    pub total_pool_value: u64,
    pub fees: u8,
    pub pool_config_bump: u8,
    pub lp_mint_auth_bump: u8,
    pub created_at: i64,
    pub is_active: bool,
}