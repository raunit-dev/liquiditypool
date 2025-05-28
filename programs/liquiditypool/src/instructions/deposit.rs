use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, mint_to, Mint, TokenAccount, TokenInterface, TransferChecked, MintTo},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use crate::state::pool_config::LiquidityPoolConfig;

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"poolconfig", pool_config.mint_a.as_ref(), pool_config.mint_b.as_ref()],
        bump = pool_config.pool_config_bump,
    )]
    pub pool_config: Account<'info, LiquidityPoolConfig>,

    // Token mints
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,
    
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = lp_mint_auth,
    )]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    // Pool vaults
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool_config,
        associated_token::token_program = token_program,
    )]
    pub vault_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = pool_config,
        associated_token::token_program = token_program,
    )]
    pub vault_token_b: InterfaceAccount<'info, TokenAccount>,

    // User token accounts
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lp_token: InterfaceAccount<'info, TokenAccount>,

    // Pyth price feeds
    pub price_feed_a: Account<'info, PriceUpdateV2>,
    pub price_feed_b: Account<'info, PriceUpdateV2>,

    // Authorities
    #[account(
        mut,
        seeds = [b"lp_mint"],
        bump
    )]
    pub lp_mint_auth: SystemAccount<'info>,

    #[account(
        seeds = [b"pool_authority", pool_config.key().as_ref()],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositLiquidity<'info> {
    pub fn deposit_liquidity(
        &mut self,
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
        price_feed_id_a: String,
        price_feed_id_b: String,
    ) -> Result<()> {
        // Validate price feeds
        let feed_id_a = get_feed_id_from_hex(&price_feed_id_a)?;
        let feed_id_b = get_feed_id_from_hex(&price_feed_id_b)?;

        // Get current prices from Pyth
        let price_a = self.price_feed_a.get_price_no_older_than(
            &Clock::get()?,
            60, // Max age in seconds
            &feed_id_a,
        )?;

        let price_b = self.price_feed_b.get_price_no_older_than(
            &Clock::get()?,
            60,
            &feed_id_b,
        )?;

        // Calculate USD values
        let value_a_usd = self.calculate_token_value_usd(
            amount_a,
            price_a.price,
            price_a.exponent,
            self.mint_a.decimals,
        )?;

        let value_b_usd = self.calculate_token_value_usd(
            amount_b,
            price_b.price,
            price_b.exponent,
            self.mint_b.decimals,
        )?;

        let total_deposit_value = value_a_usd + value_b_usd;

        // Validate minimum deposit value (optional)
        require!(total_deposit_value > 0, CustomError::InvalidDepositValue);

        // Calculate LP tokens to mint
        let lp_tokens_to_mint = if self.lp_mint.supply == 0 {
            // First deposit - mint initial LP tokens based on deposit value
            // Scale to appropriate LP token decimals
            self.scale_to_lp_decimals(total_deposit_value)?
        } else {
            // Subsequent deposits - mint proportional to existing supply
            let current_pool_value = self.calculate_current_pool_value(
                &price_a, &price_b, &feed_id_a, &feed_id_b
            )?;
            
            (self.lp_mint.supply as u128)
                .checked_mul(total_deposit_value as u128)
                .ok_or(CustomError::MathOverflow)?
                .checked_div(current_pool_value as u128)
                .ok_or(CustomError::MathOverflow)? as u64
        };

        // Slippage protection
        require!(
            lp_tokens_to_mint >= min_lp_tokens,
            CustomError::SlippageExceeded
        );

        // Transfer tokens from user to pool vaults
        self.transfer_token_a(amount_a)?;
        self.transfer_token_b(amount_b)?;

        // Mint LP tokens to user
        self.mint_lp_tokens(lp_tokens_to_mint)?;

        // Update pool state
        self.pool_config.token_a_deposits = self.pool_config.token_a_deposits
            .checked_add(amount_a)
            .ok_or(CustomError::MathOverflow)?;
            
        self.pool_config.token_b_deposits = self.pool_config.token_b_deposits
            .checked_add(amount_b)
            .ok_or(CustomError::MathOverflow)?;

        self.pool_config.total_pool_value = self.pool_config.total_pool_value
            .checked_add(total_deposit_value)
            .ok_or(CustomError::MathOverflow)?;

        msg!(
            "Deposited {} token A, {} token B. Value: ${}, LP tokens: {}",
            amount_a,
            amount_b,
            total_deposit_value,
            lp_tokens_to_mint
        );

        Ok(())
    }

    fn calculate_token_value_usd(
        &self,
        amount: u64,
        price: i64,
        price_exponent: i32,
        token_decimals: u8,
    ) -> Result<u64> {
        // Convert token amount to base units
        let token_amount_scaled = amount as f64 / 10_f64.powi(token_decimals as i32);
        
        // Convert price to actual USD value
        let price_usd = price as f64 * 10_f64.powi(price_exponent);
        
        // Calculate total USD value
        let value_usd = token_amount_scaled * price_usd;
        
        // Return as scaled integer (6 decimal places for USD)
        Ok((value_usd * 1_000_000.0) as u64)
    }

    fn calculate_current_pool_value(
        &self,
        price_a: &pyth_solana_receiver_sdk::price_update::Price,
        price_b: &pyth_solana_receiver_sdk::price_update::Price,
        feed_id_a: &[u8; 32],
        feed_id_b: &[u8; 32],
    ) -> Result<u64> {
        let vault_a_balance = self.vault_token_a.amount;
        let vault_b_balance = self.vault_token_b.amount;

        let value_a = self.calculate_token_value_usd(
            vault_a_balance,
            price_a.price,
            price_a.exponent,
            self.mint_a.decimals,
        )?;

        let value_b = self.calculate_token_value_usd(
            vault_b_balance,
            price_b.price,
            price_b.exponent,
            self.mint_b.decimals,
        )?;

        Ok(value_a + value_b)
    }

    fn scale_to_lp_decimals(&self, value_usd: u64) -> Result<u64> {
        // Scale USD value to LP token decimals
        // Assuming LP token has 6 decimals and USD value is already scaled to 6 decimals
        Ok(value_usd)
    }

    fn transfer_token_a(&mut self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.user_token_a.to_account_info(),
                mint: self.mint_a.to_account_info(),
                to: self.vault_token_a.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );
        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)
    }

    fn transfer_token_b(&mut self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.user_token_b.to_account_info(),
                mint: self.mint_b.to_account_info(),
                to: self.vault_token_b.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );
        transfer_checked(cpi_ctx, amount, self.mint_b.decimals)
    }

    fn mint_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let seeds: &[&[u8]] = &[b"lp_mint", &[self.pool_config.lp_mint_auth_bump]];
        let signer_seeds = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.lp_mint.to_account_info(),
                to: self.user_lp_token.to_account_info(),
                authority: self.lp_mint_auth.to_account_info(),
            },
            signer_seeds,
        );
        mint_to(cpi_ctx, amount)
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid deposit value")]
    InvalidDepositValue,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Price feed too old")]
    PriceFeedTooOld,
}