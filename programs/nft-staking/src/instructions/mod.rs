// Instructions module - contains all the instruction handlers for the staking program
// Each file represents a different operation users can perform

pub mod claim;
pub mod initialize_config; // Admin function to set up the global staking parameters
pub mod initialize_user_accounts; // Creates a user's personal staking account
pub mod stake; // Stakes an NFT and starts earning rewards
pub mod unstake; // Unstakes an NFT and claims earned rewards // Claims accumulated reward points as tokens

// Re-export all instruction structs and implementations
pub use claim::*;
pub use initialize_config::*;
pub use initialize_user_accounts::*;
pub use stake::*;
pub use unstake::*;
