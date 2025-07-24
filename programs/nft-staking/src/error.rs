use anchor_lang::prelude::*; // Import Anchor framework essentials

// Define custom error codes for the staking program
// These provide meaningful error messages when operations fail
#[error_code]
pub enum ErrorCode {
    #[msg("Time has not yet elapsed")] // Error message shown to users
    TimeNotElapsed, // Thrown when trying to unstake before freeze period ends

    #[msg("Maximum stake limit reached")] // Error message shown to users
    MaxStake, // Thrown when user tries to stake more NFTs than allowed

    #[msg("Arithmetic underflow")] // Error message shown to users
    Underflow, // Thrown when subtraction would result in negative number

    #[msg("Arithmetic overflow")] // Error message shown to users
    Overflow, // Thrown when addition would exceed maximum value limits
}
