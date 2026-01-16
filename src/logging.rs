use tracing::{info, debug, warn, error};
use chrono::Utc;
use std::time::Instant;

// Logging utilities for vault operations
pub struct Logger;

impl Logger {
    // Log when a vault operation starts
    pub fn log_vault_operation_start(operation: &str, user: &str, vault: &str) {
        info!(
            target: "vault_operations",
            "[START] {} | User: {} | Vault: {} | Time: {}",
            operation,
            user,
            vault,
            Utc::now().to_rfc3339()
        );
    }

    // Log when a vault operation completes successfully
    pub fn log_vault_operation_success(
        operation: &str,
        user: &str,
        vault: &str,
        duration_ms: u128,
    ) {
        info!(
            target: "vault_operations",
            "[SUCCESS] {} | User: {} | Vault: {} | Time: {}ms | At: {}",
            operation,
            user,
            vault,
            duration_ms,
            Utc::now().to_rfc3339()
        );
    }

    // Log when a vault operation fails
    pub fn log_vault_operation_error(
        operation: &str,
        user: &str,
        vault: &str,
        error: &str,
        duration_ms: u128,
    ) {
        error!(
            target: "vault_operations",
            "[ERROR] {} | User: {} | Vault: {} | Error: {} | Time: {}ms | At: {}",
            operation,
            user,
            vault,
            error,
            duration_ms,
            Utc::now().to_rfc3339()
        );
    }

    /// Log deposit transaction
    pub fn log_deposit(user: &str, amount: u64, tx_sig: &str) {
        info!(
            target: "transactions",
            "[DEPOSIT] User: {} | Amount: {} | Signature: {} | Timestamp: {}",
            user,
            amount,
            tx_sig,
            Utc::now().to_rfc3339()
        );
    }

    /// Log withdrawal transaction
    pub fn log_withdrawal(user: &str, amount: u64, tx_sig: &str) {
        info!(
            target: "transactions",
            "[WITHDRAWAL] User: {} | Amount: {} | Signature: {} | Timestamp: {}",
            user,
            amount,
            tx_sig,
            Utc::now().to_rfc3339()
        );
    }

    /// Log balance change
    pub fn log_balance_change(user: &str, old_balance: u64, new_balance: u64, reason: &str) {
        debug!(
            target: "balances",
            "[BALANCE_CHANGE] User: {} | Old: {} | New: {} | Reason: {} | Timestamp: {}",
            user,
            old_balance,
            new_balance,
            reason,
            Utc::now().to_rfc3339()
        );
    }

    /// Log reconciliation event
    pub fn log_reconciliation(vault: &str, on_chain: u64, off_chain: u64, status: &str) {
        info!(
            target: "reconciliation",
            "[RECONCILIATION] Vault: {} | On-chain: {} | Off-chain: {} | Status: {} | Timestamp: {}",
            vault,
            on_chain,
            off_chain,
            status,
            Utc::now().to_rfc3339()
        );
    }

    /// Log lock operation
    pub fn log_lock_operation(user: &str, amount: u64, reason: &str) {
        info!(
            target: "locking",
            "[LOCK] User: {} | Amount: {} | Reason: {} | Timestamp: {}",
            user,
            amount,
            reason,
            Utc::now().to_rfc3339()
        );
    }

    /// Log unlock operation
    pub fn log_unlock_operation(user: &str, amount: u64, reason: &str) {
        info!(
            target: "locking",
            "[UNLOCK] User: {} | Amount: {} | Reason: {} | Timestamp: {}",
            user,
            amount,
            reason,
            Utc::now().to_rfc3339()
        );
    }

    /// Log RPC call
    pub fn log_rpc_call(method: &str, params: &str) {
        debug!(
            target: "rpc",
            "[RPC_CALL] Method: {} | Params: {} | Timestamp: {}",
            method,
            params,
            Utc::now().to_rfc3339()
        );
    }

    /// Log RPC response
    pub fn log_rpc_response(method: &str, duration_ms: u128, success: bool) {
        if success {
            debug!(
                target: "rpc",
                "[RPC_RESPONSE_OK] Method: {} | Duration: {}ms | Timestamp: {}",
                method,
                duration_ms,
                Utc::now().to_rfc3339()
            );
        } else {
            warn!(
                target: "rpc",
                "[RPC_RESPONSE_ERROR] Method: {} | Duration: {}ms | Timestamp: {}",
                method,
                duration_ms,
                Utc::now().to_rfc3339()
            );
        }
    }

    /// Log database operation
    pub fn log_db_operation(operation: &str, table: &str, duration_ms: u128) {
        debug!(
            target: "database",
            "[DB_{}] Table: {} | Duration: {}ms | Timestamp: {}",
            operation,
            table,
            duration_ms,
            Utc::now().to_rfc3339()
        );
    }

    /// Log API request
    pub fn log_api_request(method: &str, path: &str, user: &str) {
        info!(
            target: "api",
            "[REQUEST] {} {} | User: {} | Timestamp: {}",
            method,
            path,
            user,
            Utc::now().to_rfc3339()
        );
    }

    /// Log API response
    pub fn log_api_response(method: &str, path: &str, status: u16, duration_ms: u128) {
        info!(
            target: "api",
            "[RESPONSE] {} {} | Status: {} | Duration: {}ms | Timestamp: {}",
            method,
            path,
            status,
            duration_ms,
            Utc::now().to_rfc3339()
        );
    }

    /// Log security event
    pub fn log_security_event(event_type: &str, user: &str, details: &str, severity: &str) {
        error!(
            target: "security",
            "[SECURITY][{}] Type: {} | User: {} | Details: {} | Timestamp: {}",
            severity,
            event_type,
            user,
            details,
            Utc::now().to_rfc3339()
        );
    }

    /// Log state mismatch
    pub fn log_state_mismatch(vault: &str, expected: u64, actual: u64) {
        warn!(
            target: "consistency",
            "[STATE_MISMATCH] Vault: {} | Expected: {} | Actual: {} | Timestamp: {}",
            vault,
            expected,
            actual,
            Utc::now().to_rfc3339()
        );
    }

    /// Log retry attempt
    pub fn log_retry_attempt(operation: &str, attempt: u32, reason: &str) {
        warn!(
            target: "retry",
            "[RETRY] Operation: {} | Attempt: {} | Reason: {} | Timestamp: {}",
            operation,
            attempt,
            reason,
            Utc::now().to_rfc3339()
        );
    }

    /// Log CPI call
    pub fn log_cpi_call(caller: &str, target_program: &str, instruction: &str) {
        info!(
            target: "cpi",
            "[CPI] Caller: {} | Target: {} | Instruction: {} | Timestamp: {}",
            caller,
            target_program,
            instruction,
            Utc::now().to_rfc3339()
        );
    }

    /// Log indexer event
    pub fn log_indexer_event(event_type: &str, tx_sig: &str, details: &str) {
        debug!(
            target: "indexer",
            "[EVENT] Type: {} | Signature: {} | Details: {} | Timestamp: {}",
            event_type,
            tx_sig,
            details,
            Utc::now().to_rfc3339()
        );
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    start: Instant,
    operation_name: String,
}

impl OperationTimer {
    pub fn new(operation_name: &str) -> Self {
        Self {
            start: Instant::now(),
            operation_name: operation_name.to_string(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn log_completion(&self) {
        let elapsed = self.elapsed_ms();
        info!(
            target: "performance",
            "[OPERATION_COMPLETE] {}: {}ms",
            self.operation_name,
            elapsed
        );
    }

    pub fn log_if_slow(&self, threshold_ms: u128) {
        let elapsed = self.elapsed_ms();
        if elapsed > threshold_ms {
            warn!(
                target: "performance",
                "[SLOW_OPERATION] {}: {}ms (threshold: {}ms)",
                self.operation_name,
                elapsed,
                threshold_ms
            );
        }
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        // Auto-log on drop if not explicitly logged
        let elapsed = self.elapsed_ms();
        if elapsed > 1000 {
            // Log slow operations
            warn!(
                target: "performance",
                "[OPERATION_TIMEOUT] {}: {}ms",
                self.operation_name,
                elapsed
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = OperationTimer::new("test_operation");
        assert!(timer.elapsed_ms() >= 0);
    }

    #[test]
    fn test_timer_elapsed() {
        let timer = OperationTimer::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }

    #[test]
    fn test_slow_operation_timer() {
        let timer = OperationTimer::new("slow_op");
        std::thread::sleep(std::time::Duration::from_millis(50));
        timer.log_if_slow(100); // Should not warn
        
        let timer2 = OperationTimer::new("slow_op2");
        std::thread::sleep(std::time::Duration::from_millis(150));
        timer2.log_if_slow(100); // Would warn if logger is configured
    }
}
