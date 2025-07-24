// State module - contains all account data structures for the staking program

pub mod stake_account;
pub mod stake_config; // Global configuration settings for the staking program
pub mod user_accounts; // Individual user staking data and statistics // Individual NFT stake records and metadata

// Re-export all state structures so they can be imported with use crate::state::*
pub use stake_account::*;
pub use stake_config::*;
pub use user_accounts::*;
