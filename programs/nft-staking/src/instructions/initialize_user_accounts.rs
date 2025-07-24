#![allow(unexpected_cfgs)] // Allow compiler warnings for unrecognized configuration flags

use anchor_lang::prelude::*; // Import essential Anchor framework items

use crate::state::UserAccount; // Import the user account data structure

// Account validation struct for creating a user's staking account
// This must be called before a user can stake any NFTs
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)] // Account can be modified (will pay for account creation)
    pub user: Signer<'info>, // The user wallet creating their staking account

    #[account(
        init, // Create a new account
        payer = user, // User pays the rent for their own account creation
        seeds = [b"user".as_ref(), user.key().as_ref()], // PDA using user's pubkey as seed
        bump, // Anchor finds the canonical bump seed automatically
        space = 8 + UserAccount::INIT_SPACE, // 8 bytes discriminator + UserAccount struct
    )]
    pub user_account: Account<'info, UserAccount>, // The user's staking account being created
    pub system_program: Program<'info, System>, // Solana system program for account creation
}

// Implementation block containing the instruction logic
impl<'info> Initialize<'info> {
    // Function to initialize a user's staking account with default values
    pub fn init_user(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // Set the initial data for the user's staking account
        self.user_account.set_inner(UserAccount {
            points: 0,                // Start with zero reward points
            amount_staked: 0,         // User hasn't staked any NFTs yet
            bump: bumps.user_account, // Store the PDA bump for future lookups
        });

        Ok(()) // Return success
    }
}
