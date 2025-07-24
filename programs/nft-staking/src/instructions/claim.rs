// Import custom error types and state structures
use crate::error::ErrorCode; // Fixed import path for error types
use crate::state::*; // Import all state structures
                     // Import essential Anchor and SPL Token types
use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount}; // For minting reward tokens

// Account validation struct for claiming staking rewards
// Allows users to mint reward tokens based on their accumulated points
#[derive(Accounts)]
pub struct Claim<'info> {
    /// User claiming their staking rewards
    #[account(mut)] // Account can be modified (pays transaction fees)
    pub user: Signer<'info>, // The user claiming their reward tokens

    /// User's staking account (holds accumulated points)
    #[account(
        mut, // Account will be modified (points reset to zero after claiming)
        seeds = [b"user", user.key.as_ref()], // User's staking account PDA
        bump = user_account.bump // Use stored bump from user account
    )]
    pub user_account: Account<'info, UserAccount>, // User's staking statistics and points

    /// Global staking configuration
    #[account(
        seeds = [b"config"], // Global config PDA seed
        bump = config.bump // Use stored bump from config
    )]
    pub config: Account<'info, StakeConfig>, // Global staking configuration

    /// Reward token mint
    #[account(
        mut, // Account will be modified (tokens will be minted)
        seeds = [b"rewards", config.key().as_ref()], // Rewards mint PDA using config as seed
        bump = config.rewards_bump // Use stored bump from config
    )]
    pub reward_mint: Account<'info, Mint>, // The mint for reward tokens

    /// User's associated token account to receive reward tokens
    #[account(
        mut, // Account will be modified (receives newly minted tokens)
        associated_token::mint = reward_mint, // Must be ATA for the reward token mint
        associated_token::authority = user // Must be owned by the user
    )]
    pub user_reward_ata: Account<'info, TokenAccount>, // User's token account for reward tokens

    /// Programs and sysvars
    pub token_program: Program<'info, Token>, // SPL Token program for minting operations
}

// Implementation block containing the claiming logic
impl<'info> Claim<'info> {
    // Function to claim accumulated reward points as tokens
    pub fn claim(&mut self) -> Result<()> {
        let amount = self.user_account.points; // Get user's accumulated points

        // Don't allow claiming if no points
        require!(amount > 0, ErrorCode::MaxStake); // Reusing MaxStake error for no rewards validation

        // Mint reward tokens to user's ATA
        let seeds: &[&[u8]] = &[b"config", &[self.config.bump]]; // Config PDA seeds for signing
        let signer = &[seeds]; // Format for CPI signing

        // Prepare accounts for minting tokens
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(), // The reward token mint
            to: self.user_reward_ata.to_account_info(), // User's token account to receive tokens
            authority: self.config.to_account_info(), // Config PDA has mint authority
        };

        // Create CPI context with signer (config PDA signs on behalf of program)
        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer);

        // Mint tokens equal to accumulated points (converted to u64)
        mint_to(cpi_ctx, amount.into())?; // Mint reward tokens to user

        // Reset user points after claiming
        self.user_account.points = 0; // Clear points since they've been claimed as tokens

        Ok(()) // Return success
    }
}
