#![allow(unexpected_cfgs)] // Allow compiler warnings for unrecognized configuration flags

// Import custom error types and state structures
use crate::error::ErrorCode;
use crate::state::*;
// Import essential Anchor and SPL Token types
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, // For associated token account operations
    token::{transfer, Mint, Token, TokenAccount, Transfer}, // For token transfer operations
};

// Account validation struct for unstaking an NFT
// Defines all accounts needed and their validation constraints
#[derive(Accounts)]
pub struct Unstake<'info> {
    /// User unstaking their NFT
    #[account(mut)] // Account can be modified (receives rent from closed stake account)
    pub user: Signer<'info>, // The user unstaking their NFT

    /// User account tracking staked amount and points
    #[account(
        mut, // Account will be modified (amount_staked decreases, points increase)
        seeds = [b"user", user.key.as_ref()], // User's staking account PDA
        bump = user_account.bump // Use stored bump from user account
    )]
    pub user_account: Account<'info, UserAccount>, // User's overall staking statistics

    /// Global staking config
    #[account(
        mut, // Account might be modified (though not in current implementation)
        seeds = [b"config"], // Global config PDA seed
        bump = config.bump // Use stored bump from config
    )]
    pub config: Account<'info, StakeConfig>, // Global staking configuration

    /// NFT mint being unstaked
    pub nft_mint: Account<'info, Mint>, // The NFT mint being unstaked

    /// Stake record for this NFT, to be closed after unstaking
    #[account(
        mut, // Account will be modified (closed and rent returned)
        seeds = [b"stake", user.key.as_ref(), nft_mint.key().as_ref()], // Stake account PDA
        bump = stake_account.bump, // Use stored bump from stake account
        close = user  // Return rent to user when account is closed
    )]
    pub stake_account: Account<'info, StakeAccount>, // Individual stake record being closed

    /// Vault holding the staked NFT
    #[account(
        mut, // Account will be modified (NFT will be transferred out)
        seeds = [b"vault", nft_mint.key().as_ref()], // Vault PDA for this specific NFT
        bump, // Anchor finds the canonical bump automatically
    )]
    pub vault_ata: Account<'info, TokenAccount>, // Token account that held the staked NFT

    /// User's token account to receive NFT
    #[account(
        mut, // Account will be modified (receives the unstaked NFT)
        associated_token::mint = nft_mint, // Must be ATA for the specific NFT mint
        associated_token::authority = user, // Must be owned by the user
    )]
    pub user_nft_ata: Account<'info, TokenAccount>, // User's token account to receive NFT back

    /// Programs
    pub token_program: Program<'info, Token>, // SPL Token program for transfers
    pub associated_token_program: Program<'info, AssociatedToken>, // For ATA operations
    pub system_program: Program<'info, System>,                    // For account operations
    pub rent: Sysvar<'info, Rent>,                                 // Rent sysvar for calculations
    pub clock: Sysvar<'info, Clock>, // Clock sysvar for timestamp verification
}

// Implementation block containing the unstaking logic
impl<'info> Unstake<'info> {
    // Function to unstake an NFT and claim earned rewards
    pub fn unstake(&mut self) -> Result<()> {
        // Check that the freeze period has passed
        let now = Clock::get()?.unix_timestamp; // Get current timestamp
        require!(
            now - self.stake_account.staked_at >= self.config.freeze_period as i64, // Check if enough time has passed
            ErrorCode::TimeNotElapsed // Error if freeze period not over
        );

        // Ensure user has at least one NFT staked
        require!(
            self.user_account.amount_staked > 0, // Verify user has staked NFTs
            ErrorCode::MaxStake                  // Reusing MaxStake error for this validation
        );

        // Decrease the user's staked NFT count
        self.user_account.amount_staked = self
            .user_account
            .amount_staked
            .checked_sub(1) // Safely subtract 1 to prevent underflow
            .ok_or(ErrorCode::Underflow)?; // Return error if underflow would occur

        // Increase user's reward points (this NFT's reward)
        self.user_account.points = self
            .user_account
            .points
            .checked_add(self.config.points_per_stake as u32) // Safely add reward points
            .ok_or(ErrorCode::Overflow)?; // Return error if overflow would occur

        // Generate signer seeds for config PDA
        let seeds: &[&[u8]] = &[b"config", &[self.config.bump]]; // Config PDA seeds
        let signer: &[&[&[u8]]; 1] = &[seeds]; // Format for CPI signing

        // Transfer the NFT token from vault ATA back to user's wallet
        let cpi_accounts = Transfer {
            from: self.vault_ata.to_account_info(), // Source: vault holding the NFT
            to: self.user_nft_ata.to_account_info(), // Destination: user's token account
            authority: self.config.to_account_info(), // Authority: config PDA (vault owner)
        };

        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer);

        // Only 1 NFT is transferred
        transfer(cpi_ctx, 1)?; // Transfer exactly 1 token (the NFT)

        Ok(()) // Return success (stake account automatically closed due to close constraint)
    }
}
