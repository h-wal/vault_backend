//! Vault Backend - Solana vault management system
//!
//! This library provides functionality for managing collateral vaults on Solana,
//! including initialization, deposits, withdrawals, and balance tracking.

pub mod access_control;
pub mod api;
pub mod config;
pub mod cpi_manager;
pub mod db;
pub mod error_handling;
pub mod idl;
pub mod indexer;
pub mod logging;
pub mod reconciliation;
pub mod states;
pub mod transaction_builder;
pub mod vault_manager;

// Re-export commonly used types
pub use config::Config;
pub use error_handling::{RetryConfig, VaultError};
pub use states::CollateralVault;
pub use transaction_builder::TransactionBuilder;
pub use vault_manager::VaultManager;
