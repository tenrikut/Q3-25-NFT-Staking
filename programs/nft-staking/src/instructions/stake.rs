#![allow(unexpected_cfgs)] // Allow compiler warnings for unrecognized configuration flags

use anchor_lang::prelude::*; // Import essential Anchor framework items
                             // Import SPL Token and Metadata program types for NFT handling
use anchor_spl::{
    metadata::{mpl_token_metadata, MasterEditionAccount, Metadata, MetadataAccount},
    token::{approve, Approve, Mint, Token, TokenAccount},
};
// Import Metaplex instruction for freezing delegated NFTs
use mpl_token_metadata::instructions::{
    FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts,
};
// Import program state structures
use crate::state::stake_account::StakeAccount;
use crate::state::stake_config::StakeConfig;
use crate::state::user_accounts::UserAccount;
// Import custom error types
use crate::error::ErrorCode;

// Account validation struct for staking an NFT
// Defines all accounts needed and their validation constraints
#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)] // Account can be modified (pays for stake account creation)
    pub user: Signer<'info>, // The user staking their NFT

    pub mint: Account<'info, Mint>, // The NFT mint being staked
    pub collection_mint: Account<'info, Mint>, // The collection this NFT belongs to

    #[account(
        mut, // Account will be modified (approval will be set)
        associated_token::mint = mint, // Must be ATA for the specific NFT mint
        associated_token::authority = user, // Must be owned by the user
    )]
    pub mint_ata: Account<'info, TokenAccount>, // User's token account holding the NFT

    #[account(
        seeds = [
            b"metadata", // Metaplex metadata PDA seed
            metadata_program.key().as_ref(), // Metadata program ID
            mint.key().as_ref() // NFT mint address
        ],
        seeds::program = metadata_program.key(), // Use metadata program for PDA derivation
        bump, // Anchor finds the canonical bump automatically
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(), // Verify NFT belongs to expected collection
        constraint = metadata.collection.as_ref().unwrap().verified == true, // Verify collection is verified by creator
    )]
    pub metadata: Account<'info, MetadataAccount>, // NFT metadata account

    #[account(
        seeds = [
            b"metadata", // Metaplex metadata PDA seed
            metadata_program.key().as_ref(), // Metadata program ID  
            mint.key().as_ref(), // NFT mint address
            b"edition" // Master edition seed
        ],
        seeds::program = metadata_program.key(), // Use metadata program for PDA derivation
        bump, // Anchor finds the canonical bump automatically
    )]
    pub edition: Account<'info, MasterEditionAccount>, // NFT master edition account (proves it's an NFT)

    #[account(
        seeds = [b"config".as_ref()], // Global config PDA seed
        bump = config.bump // Use stored bump from config
    )]
    pub config: Account<'info, StakeConfig>, // Global staking configuration

    #[account(
        init, // Create new stake record for this NFT
        payer = user, // User pays for stake account creation
        space = 8 + StakeAccount::INIT_SPACE, // 8 bytes discriminator + struct size
        seeds = [b"stake".as_ref(), mint.key().as_ref(), config.key().as_ref()], // Unique PDA per NFT per config
        bump, // Anchor finds the canonical bump automatically
    )]
    pub stake_account: Account<'info, StakeAccount>, // Individual stake record being created

    #[account(
        mut, // Account will be modified (amount_staked will increase)
        seeds = [b"user".as_ref(), user.key().as_ref()], // User's staking account PDA
        bump = user_account.bump, // Use stored bump from user account
    )]
    pub user_account: Account<'info, UserAccount>, // User's overall staking statistics

    // Required Solana programs
    pub system_program: Program<'info, System>, // For account creation
    pub token_program: Program<'info, Token>,   // For token operations
    pub metadata_program: Program<'info, Metadata>, // For NFT metadata operations
}

// Implementation block containing the staking logic
impl<'info> Stake<'info> {
    // Function to stake an NFT and start earning rewards
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        // Verify user hasn't exceeded their staking limit
        require!(
            self.user_account.amount_staked < self.config.max_stake,
            ErrorCode::MaxStake
        );

        // Create the stake record with current timestamp
        self.stake_account.set_inner(StakeAccount {
            owner: self.user.key(),                  // Store who staked this NFT
            mint: self.mint.key(),                   // Store which NFT was staked
            staked_at: Clock::get()?.unix_timestamp, // Store when it was staked (for freeze period)
            bump: bumps.stake_account,               // Store PDA bump for future lookups
        });

        // Approve the stake account as delegate for the NFT (allows program to control it)
        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();
        let cpi_accounts: Approve<'_> = Approve {
            to: self.mint_ata.to_account_info(), // The token account holding the NFT
            delegate: self.stake_account.to_account_info(), // The stake account becomes delegate
            authority: self.user.to_account_info(), // User authorizes this delegation
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        approve(cpi_ctx, 1)?; // Approve for exactly 1 token (the NFT)

        // Generate PDA signer seeds for the stake account to sign on behalf of the program
        let seeds: &[&[u8]; 4] = &[
            b"stake",                                   // Stake PDA seed
            self.mint.to_account_info().key.as_ref(),   // NFT mint address
            self.config.to_account_info().key.as_ref(), // Config address
            &[self.stake_account.bump],                 // PDA bump
        ];
        let signer_seeds = &[&seeds[..]]; // Format for CPI signing

        // Prepare accounts for freezing the NFT (prevents transfers while staked)
        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let edition = &self.edition.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        // Freeze the NFT so it cannot be transferred while staked
        FreezeDelegatedAccountCpi::new(
            metadata_program,
            FreezeDelegatedAccountCpiAccounts {
                delegate,      // Stake account that now controls the NFT
                token_account, // Token account holding the NFT
                edition,       // Master edition account
                mint,          // NFT mint
                token_program, // SPL Token program
            },
        )
        .invoke_signed(signer_seeds)?; // Sign with stake account PDA

        // Update user's staking statistics
        self.user_account.amount_staked += 1; // Increment their staked NFT count

        Ok(()) // Return success
    }
}
