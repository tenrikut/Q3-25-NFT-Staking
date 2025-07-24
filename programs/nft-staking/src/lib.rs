//declare_id!("6YvXnSvATQbKDtaoSxpenuZmsYwTnFW2ie4CarKpX86r");

#![allow(unexpected_cfgs)] // Allow compiler warnings for configuration flags that might not be recognized
#![allow(deprecated)] // Allow usage of deprecated features without warnings

// Module declarations - these tell Rust about other files in the project
pub mod constants; // Contains program-wide constants like seeds
pub mod error; // Contains custom error types for the program
pub mod instructions; // Contains all instruction handlers (initialize, stake, etc.)
pub mod state; // Contains account data structures

use anchor_lang::prelude::*; // Import all essential Anchor framework items

// Re-export all items from modules so they can be used throughout the program
pub use constants::*;
pub use instructions::*;
pub use state::*;

// Declare the program ID - this is the unique identifier for this Solana program
// This ID must match what's deployed on-chain
declare_id!("6YvXnSvATQbKDtaoSxpenuZmsYwTnFW2ie4CarKpX86r");
// The main program module - contains all public instruction handlers
#[program]
pub mod nft_staking {
    use super::*; // Import everything from the parent scope

    // Initialize the global staking configuration (admin-only function)
    // Parameters: points earned per stake, maximum NFTs per user, freeze time in seconds
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        points_per_stake: u8,
        max_stake: u8,
        freeze_period: u32,
    ) -> Result<()> {
        // Delegate to the instruction handler with account context and PDA bumps
        ctx.accounts
            .initialize_config(points_per_stake, max_stake, freeze_period, &ctx.bumps)
    }

    // Initialize a user's staking account (creates their personal staking data)
    pub fn initialize_user(ctx: Context<Initialize>) -> Result<()> {
        // Delegate to the instruction handler with PDA bumps for account creation
        ctx.accounts.init_user(&ctx.bumps)
    }

    // Stake an NFT (locks it and starts earning rewards)
    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        // Delegate to the instruction handler with PDA bumps for new stake account
        ctx.accounts.stake(&ctx.bumps)
    }

    // Unstake an NFT (unlocks it and claims accumulated rewards)
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        // Delegate to the instruction handler (no bumps needed as account is closed)
        ctx.accounts.unstake()
    }

    // Claim accumulated reward points as mintable tokens
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        // Delegate to the instruction handler (no bumps needed as no accounts created)
        ctx.accounts.claim()
    }
}
