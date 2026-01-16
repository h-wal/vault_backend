# Quick Code Examples

## Error Handling

### Error Handling Module
`Module`: `vault_backend::error_handling`

This handles errors and retries:

```rust
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

pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}
```

Functions:
```rust
// Retry an async operation
pub async fn retry_with_backoff<F, Fut, T>(
    config: RetryConfig,
    f: F,
) -> Result<T>

// Retry a sync operation
pub fn retry_sync<F, T>(config: RetryConfig, f: F) -> Result<T>

// Check if error can be retried
pub fn is_retryable_error(error: &anyhow::Error) -> bool
```

Usage:
```rust
use vault_backend::error_handling::{RetryConfig, retry_with_backoff};

let config = RetryConfig::default();
retry_with_backoff(config, || async {
    // Try some operation
    rpc_call().await
}).await?
```

---

## Access Control

### Authorization & Security
`Module`: `vault_backend::access_control`

Authorization and security monitoring:

```rust
pub struct AccessControlManager { ... }

pub enum SecurityEventType {
    UnauthorizedAccessAttempt,
    SuspiciousWithdrawal,
    RapidTransactionSequence,
    LargeUnexpectedTransfer,
    AccountStateChange,
}

pub enum AlertSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub user: String,
    pub vault: String,
    pub timestamp: DateTime<Utc>,
    pub details: String,
    pub severity: AlertSeverity,
}
```

Methods:
```rust
// Authorization
pub async fn authorize_user(&self, vault: &str, user: &str)
pub async fn is_authorized(&self, vault: &str, user: &str) -> bool

// Security Events
pub async fn record_unauthorized_attempt(...)

pub async fn record_suspicious_withdrawal(...)
pub async fn record_rapid_transactions(...)

// Monitoring
pub async fn get_security_events(&self) -> Vec<SecurityEvent>
pub async fn get_alerts_by_severity(&self, min_severity: AlertSeverity)
pub async fn is_user_blocked(&self, user: &str) -> bool
```

**Usage**:
```rust
use vault_backend::access_control::AccessControlManager;

let acm = AccessControlManager::new();

// Grant access
acm.authorize_user("vault1", "user1").await?;

// Check access
if acm.is_authorized("vault1", "user1").await {
    // Allow operation
}

// Detect threats
acm.record_suspicious_withdrawal("user", "vault", 1_000_000_000, 100_000_000).await?;

// Get alerts
let critical = acm.get_alerts_by_severity(AlertSeverity::Critical).await;
```

---

### 3. Logging Module
**Module**: `vault_backend::logging`

**Purpose**: Structured logging with performance tracking

**Key Types**:
```rust
pub struct Logger;

pub struct OperationTimer {
    start: Instant,
    operation_name: String,
}
```

**Logging Methods**:
```rust
// Vault Operations
Logger::log_vault_operation_start(operation, user, vault)
Logger::log_vault_operation_success(operation, user, vault, duration_ms)
Logger::log_vault_operation_error(operation, user, vault, error, duration_ms)

// Transactions
Logger::log_deposit(user, amount, tx_sig)
Logger::log_withdrawal(user, amount, tx_sig)

// State & Consistency
Logger::log_balance_change(user, old, new, reason)
Logger::log_state_mismatch(vault, expected, actual)
Logger::log_reconciliation(vault, on_chain, off_chain, status)

// Performance
Logger::log_rpc_call(method, params)
Logger::log_rpc_response(method, duration_ms, success)
Logger::log_db_operation(operation, table, duration_ms)

// Security
Logger::log_security_event(event_type, user, details, severity)

// API
Logger::log_api_request(method, path, user)
Logger::log_api_response(method, path, status, duration_ms)

// Other
Logger::log_lock_operation(user, amount, reason)
Logger::log_unlock_operation(user, amount, reason)
Logger::log_cpi_call(caller, target_program, instruction)
Logger::log_indexer_event(event_type, tx_sig, details)
Logger::log_retry_attempt(operation, attempt, reason)
```

**Timer Usage**:
```rust
use vault_backend::logging::OperationTimer;

let timer = OperationTimer::new("expensive_operation");
// ... do work ...
timer.log_completion();
timer.log_if_slow(1000); // Log if > 1 second
```

**Configure Logging**:
```bash
export RUST_LOG=info,vault_backend=debug,vault_backend::logging=trace
cargo run
```

**Log Targets**:
- `vault_operations`: Vault operations
- `transactions`: Transaction logs
- `balances`: Balance changes
- `reconciliation`: Reconciliation events
- `locking`: Lock/unlock operations
- `rpc`: RPC calls
- `database`: DB operations
- `api`: API requests/responses
- `security`: Security events
- `consistency`: State consistency
- `retry`: Retry attempts
- `cpi`: Cross-program calls
- `indexer`: Indexer events
- `performance`: Performance metrics

---

## Test Files

### Vault Manager Tests
**File**: `src/vault_manager_tests.rs`

**Test Coverage**:
- Vault manager creation
- PDA derivation (deterministic)
- Different users different PDAs
- Deposit request validation
- Withdrawal validation
- Balance tracking
- Multiple deposits
- Lock/unlock sequences
- Zero amount rejection
- Transaction history ordering
- State consistency

**Run**:
```bash
cargo test vault_manager_tests
```

---

## Environment Setup

### Set Logging Level
```bash
export RUST_LOG=info,vault_backend=debug
```

### Debug Mode
```bash
export RUST_LOG=debug,vault_backend=trace
cargo run
```

### Production Mode
```bash
export RUST_LOG=warn
cargo run --release
```

---

## Common Patterns

### Safe Operation with Retry
```rust
use vault_backend::error_handling::{RetryConfig, retry_with_backoff};

let config = RetryConfig {
    max_attempts: 5,
    initial_delay_ms: 100,
    max_delay_ms: 10000,
    backoff_multiplier: 2.0,
};

let result = retry_with_backoff(config, || async {
    rpc_client.confirm_transaction(&tx).await
}).await?;
```

### Authorization Check
```rust
let acm = AccessControlManager::new();

// Setup
acm.authorize_user("vault1", "user1").await?;

// Operation
if !acm.is_authorized("vault1", "user1").await {
    acm.record_unauthorized_attempt("user1", "vault1", "operation").await?;
    return Err("Unauthorized");
}
```

### Performance Tracking
```rust
use vault_backend::logging::{Logger, OperationTimer};

Logger::log_vault_operation_start("deposit", "user1", "vault1");

let timer = OperationTimer::new("vault_operation");
// ... perform operation ...

if let Ok(sig) = result {
    Logger::log_vault_operation_success("deposit", "user1", "vault1", timer.elapsed_ms());
} else {
    Logger::log_vault_operation_error("deposit", "user1", "vault1", &err.to_string(), timer.elapsed_ms());
}
```

### Security Monitoring
```rust
// Detect suspicious activity
acm.record_rapid_transactions("user1", "vault1", 10, 5).await?;

// Get critical alerts
let alerts = acm.get_alerts_by_severity(AlertSeverity::Critical).await;
for alert in alerts {
    eprintln!("CRITICAL: {} - {}", alert.event_type, alert.details);
}
```

---

## API Documentation

**Main Resource**: `API_DOCUMENTATION.md`

**Quick Endpoints**:
- `POST /vault/initialize` - Create vault
- `POST /vault/deposit` - Deposit funds
- `POST /vault/withdraw` - Withdraw funds
- `GET /vault/balance/:user` - Get balance
- `GET /vault/transactions/:user` - Get history
- `GET /vault/tvl` - Get total locked value
- `WS /ws/vaults` - Real-time updates

---

## Troubleshooting

### Retries Not Working?
Check that error is classified as retryable:
```rust
use vault_backend::error_handling::is_retryable_error;

if is_retryable_error(&error) {
    // Will be retried
}
```

### Access Control Not Found?
Verify authorization is set:
```rust
acm.authorize_user("vault", "user").await?;
```

### Logs Not Showing?
Set log level:
```bash
export RUST_LOG=debug
```

---

## Performance Tips

1. Use async retry for I/O operations
2. Use sync retry for CPU-bound operations
3. Monitor slow operations with timers
4. Set appropriate timeout thresholds
5. Use structured logging for analytics

---

## Security Reminders

✅ Always check authorization before operations
✅ Log all security events
✅ Monitor alert severity levels
✅ Implement rate limiting at API level
✅ Use exponential backoff for retries
✅ Track failed attempts
✅ Block users after multiple failures

---

**Last Updated**: January 15, 2026
