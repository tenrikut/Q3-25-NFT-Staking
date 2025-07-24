use anchor_lang::prelude::*; // Import Anchor framework essentials

// Individual NFT stake record - created each time an NFT is staked
// Tracks when and by whom each specific NFT was staked
#[account] // Marks this as an Anchor account that can be stored on-chain
#[derive(InitSpace)] // Automatically calculates space needed for account storage
pub struct StakeAccount {
    pub owner: Pubkey,  // The wallet address that staked this NFT
    pub mint: Pubkey,   // The mint address of the specific NFT that's staked
    pub staked_at: i64, // Unix timestamp when this NFT was staked (for freeze period)
    pub bump: u8,       // PDA bump seed for this stake account
}
