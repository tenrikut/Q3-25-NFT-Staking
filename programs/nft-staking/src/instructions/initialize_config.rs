#![allow(unexpected_cfgs)] // Allow compiler warnings for unrecognized configuration flags

use anchor_lang::prelude::*; // Import essential Anchor framework items
use anchor_spl::token::{Mint, Token}; // Import SPL Token program types

use crate::state::StakeConfig; // Import the global configuration structure

// Account validation struct for initializing the staking program configuration
// This defines what accounts must be provided and how they should be validated
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)] // Account can be modified (will pay for account creation)
    pub admin: Signer<'info>, // The admin wallet that's setting up the program

    #[account(
        init, // Create a new account
        payer = admin, // Admin pays the rent for account creation
        seeds = [b"config".as_ref()], // PDA seed to generate deterministic address
        bump, // Anchor finds the canonical bump seed automatically
        space = 8 + StakeConfig::INIT_SPACE, // 8 bytes for discriminator + struct size
    )]
    pub config: Account<'info, StakeConfig>, // The global config account being created

    #[account(
        init_if_needed, // Create only if account doesn't exist yet
        payer = admin, // Admin pays for account creation if needed
        seeds = [b"rewards", config.key().as_ref()], // PDA using config address as seed
        bump, // Anchor finds the canonical bump seed automatically
        mint::decimals = 6, // Reward token will have 6 decimal places
        mint::authority = config, // Config PDA will be the mint authority
    )]
    pub rewards_mint: Account<'info, Mint>, // Mint for reward tokens users can claim
    pub system_program: Program<'info, System>, // Solana system program for account creation
    pub token_program: Program<'info, Token>,   // SPL Token program for mint operations
}

// Implementation block containing the actual instruction logic
impl<'info> InitializeConfig<'info> {
    // Function to initialize the global staking configuration
    pub fn initialize_config(
        &mut self,
        points_per_stake: u8,
        max_stake: u8,
        freeze_period: u32,
        bumps: &InitializeConfigBumps,
    ) -> Result<()> {
        // Set the configuration data in the newly created account
        self.config.set_inner(StakeConfig {
            points_per_stake,                 // How many points earned per staking period
            max_stake,                        // Maximum NFTs a user can stake at once
            freeze_period,                    // Minimum time NFTs must stay staked (seconds)
            rewards_bump: bumps.rewards_mint, // Store the rewards mint PDA bump
            bump: bumps.config,               // Store this config account's PDA bump
        });

        Ok(()) // Return success
    }
}
