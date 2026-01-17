use std::time::Duration;
use anyhow::Result;
use tracing::warn;

// Config for retry logic - controls how many times we retry and how long we wait
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}
// Error types specific to vault operations
#[derive(Debug)]
pub enum VaultError {
    InsufficientBalance { required: u64, available: u64 },
    UnauthorizedAccess { user: String, vault: String },
    TransactionFailed { reason: String },
    RpcConnectionError { endpoint: String },
    AccountNotFound { account: String },
    InvalidAmount { amount: u64 },
    StateMismatch { expected: String, actual: String },
    LockingError { reason: String },
    SerializationError { reason: String },
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::InsufficientBalance { required, available } => {
                write!(f, "Not enough balance: need {} but only have {}", required, available)
            }
            VaultError::UnauthorizedAccess { user, vault } => {
                write!(f, "User {} doesn't have access to vault {}", user, vault)
            }
            VaultError::TransactionFailed { reason } => {
                write!(f, "Transaction failed: {}", reason)
            }
            VaultError::RpcConnectionError { endpoint } => {
                write!(f, "Can't connect to RPC at {}", endpoint)
            }
            VaultError::AccountNotFound { account } => {
                write!(f, "Account {} doesn't exist", account)
            }
            VaultError::InvalidAmount { amount } => {
                write!(f, "Invalid amount: {}", amount)
            }
            VaultError::StateMismatch { expected, actual } => {
                write!(f, "State mismatch: expected {} but got {}", expected, actual)
            }
            VaultError::LockingError { reason } => {
                write!(f, "Can't lock collateral: {}", reason)
            }
            VaultError::SerializationError { reason } => {
                write!(f, "Serialization error: {}", reason)
            }
        }
    }
}

impl std::error::Error for VaultError {}

// Check if an error is worth retrying
// Network errors should be retried, but permission errors should not
pub fn is_retryable_error(error: &anyhow::Error) -> bool {
    let error_msg = error.to_string().to_lowercase();

    error_msg.contains("timeout")
        || error_msg.contains("connection")
        || error_msg.contains("temporarily")
        || error_msg.contains("unavailable")
        || error_msg.contains("rate limit")
}

// Retry an async operation with exponential backoff
// Waits longer between each attempt
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        attempt += 1;

        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= config.max_attempts || !is_retryable_error(&e) {
                    return Err(e);
                }

                warn!(
                    "Attempt {} failed: {}. Waiting {}ms before retry...",
                    attempt,
                    e,
                    delay
                );

                tokio::time::sleep(Duration::from_millis(delay)).await;

                delay = ((delay as f64) * config.backoff_multiplier) as u64;
                delay = delay.min(config.max_delay_ms);
            }
        }
    }
}

// Sync version of retry for non-async code
pub fn retry_sync<F, T>(
    config: RetryConfig,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        attempt += 1;

        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= config.max_attempts || !is_retryable_error(&e) {
                    return Err(e);
                }

                warn!(
                    "Attempt {} failed: {}. Waiting {}ms before retry...",
                    attempt,
                    e,
                    delay
                );

                std::thread::sleep(Duration::from_millis(delay));

                delay = ((delay as f64) * config.backoff_multiplier) as u64;
                delay = delay.min(config.max_delay_ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
    }

    #[test]
    fn test_insufficient_balance_error() {
        let err = VaultError::InsufficientBalance {
            required: 1000,
            available: 500,
        };
        let msg = err.to_string();
        assert!(msg.contains("1000"));
        assert!(msg.contains("500"));
    }

    #[test]
    fn test_unauthorized_access_error() {
        let err = VaultError::UnauthorizedAccess {
            user: "user1".to_string(),
            vault: "vault1".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("user1"));
        assert!(msg.contains("vault1"));
    }

    #[test]
    fn test_is_retryable_error_timeout() {
        use anyhow::anyhow;
        let err = anyhow!("Request timeout");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn test_is_retryable_error_connection() {
        use anyhow::anyhow;
        let err = anyhow!("Connection refused");
        assert!(is_retryable_error(&err));
    }

    #[test]
    fn test_is_not_retryable_error() {
        use anyhow::anyhow;
        let err = anyhow!("Invalid account");
        assert!(!is_retryable_error(&err));
    }
}
