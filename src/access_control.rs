use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use tracing::{warn, error};

// Different types of security issues we monitor
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityEventType {
    UnauthorizedAccessAttempt,
    SuspiciousWithdrawal,
    RapidTransactionSequence,
    LargeUnexpectedTransfer,
    AccountStateChange,
}

// Log entry for a security event
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub user: String,
    pub vault: String,
    pub timestamp: DateTime<Utc>,
    pub details: String,
    pub severity: AlertSeverity,
}

// How serious a security event is
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

// Manages who can access which vaults and monitors for suspicious activity
pub struct AccessControlManager {
    authorized_users: Arc<RwLock<HashMap<String, Vec<String>>>>, // vault -> users
    security_events: Arc<RwLock<Vec<SecurityEvent>>>,
    failed_attempts: Arc<RwLock<HashMap<String, u32>>>, // user -> failed attempts
}

impl AccessControlManager {
    pub fn new() -> Self {
        Self {
            authorized_users: Arc::new(RwLock::new(HashMap::new())),
            security_events: Arc::new(RwLock::new(Vec::new())),
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Allow a user to access a specific vault
    pub async fn authorize_user(&self, vault: &str, user: &str) -> anyhow::Result<()> {
        let mut authorized = self.authorized_users.write().await;
        authorized
            .entry(vault.to_string())
            .or_insert_with(Vec::new)
            .push(user.to_string());

        tracing::info!("User {} added to vault {}", user, vault);
        Ok(())
    }

    // Check if a user is allowed to access a vault
    pub async fn is_authorized(&self, vault: &str, user: &str) -> bool {
        let authorized = self.authorized_users.read().await;
        authorized
            .get(vault)
            .map(|users| users.contains(&user.to_string()))
            .unwrap_or(false)
    }

    // Log when someone tries to access a vault they're not authorized for
    pub async fn record_unauthorized_attempt(
        &self,
        user: &str,
        vault: &str,
        details: &str,
    ) -> anyhow::Result<()> {
        let event = SecurityEvent {
            event_type: SecurityEventType::UnauthorizedAccessAttempt,
            user: user.to_string(),
            vault: vault.to_string(),
            timestamp: Utc::now(),
            details: details.to_string(),
            severity: AlertSeverity::High,
        };

        self.security_events.write().await.push(event.clone());

        let mut failed = self.failed_attempts.write().await;
        let attempt_count = failed.entry(user.to_string()).or_insert(0);
        *attempt_count += 1;

        warn!(
            "SECURITY: {} tried to access {} unauthorized. Info: {}",
            user, vault, details
        );

        // If someone tries too many times, that's a bigger issue
        if *attempt_count >= 3 {
            error!(
                "ALERT: {} has made {} failed access attempts. Suspicious activity!",
                user, attempt_count
            );
        }

        Ok(())
    }

    // Log when a withdrawal looks unusual
    pub async fn record_suspicious_withdrawal(
        &self,
        user: &str,
        vault: &str,
        amount: u64,
        average_withdrawal: u64,
    ) -> anyhow::Result<()> {
        let event = SecurityEvent {
            event_type: SecurityEventType::SuspiciousWithdrawal,
            user: user.to_string(),
            vault: vault.to_string(),
            timestamp: Utc::now(),
            details: format!(
                "Withdrawal: {} (usually around {})",
                amount, average_withdrawal
            ),
            severity: if amount > average_withdrawal * 10 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Medium
            },
        };

        self.security_events.write().await.push(event);

        warn!(
            "SECURITY: Unusual withdrawal. User: {}, Vault: {}, Amount: {}",
            user, vault, amount
        );

        Ok(())
    }

    // Log when someone does many transactions in a short time
    pub async fn record_rapid_transactions(
        &self,
        user: &str,
        vault: &str,
        transaction_count: u32,
        time_window_secs: u64,
    ) -> anyhow::Result<()> {
        let event = SecurityEvent {
            event_type: SecurityEventType::RapidTransactionSequence,
            user: user.to_string(),
            vault: vault.to_string(),
            timestamp: Utc::now(),
            details: format!(
                "{} transactions in {} seconds",
                transaction_count, time_window_secs
            ),
            severity: AlertSeverity::High,
        };

        self.security_events.write().await.push(event);

        warn!(
            "SECURITY: Rapid transaction sequence detected. User: {}, Count: {}",
            user, transaction_count
        );

        Ok(())
    }

    /// Get all security events
    pub async fn get_security_events(&self) -> Vec<SecurityEvent> {
        self.security_events.read().await.clone()
    }

    /// Get security events for specific severity level or higher
    pub async fn get_alerts_by_severity(&self, min_severity: AlertSeverity) -> Vec<SecurityEvent> {
        self.security_events
            .read()
            .await
            .iter()
            .filter(|e| e.severity >= min_severity)
            .cloned()
            .collect()
    }

    /// Clear failed attempts for user (after successful action)
    pub async fn clear_failed_attempts(&self, user: &str) -> anyhow::Result<()> {
        self.failed_attempts.write().await.remove(user);
        Ok(())
    }

    /// Get failed attempt count for user
    pub async fn get_failed_attempts(&self, user: &str) -> u32 {
        self.failed_attempts
            .read()
            .await
            .get(user)
            .copied()
            .unwrap_or(0)
    }

    /// Block user if too many failed attempts
    pub async fn is_user_blocked(&self, user: &str) -> bool {
        self.get_failed_attempts(user).await >= 5
    }
}

impl Default for AccessControlManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authorize_user() {
        let acm = AccessControlManager::new();
        acm.authorize_user("vault1", "user1").await.unwrap();

        assert!(acm.is_authorized("vault1", "user1").await);
        assert!(!acm.is_authorized("vault1", "user2").await);
    }

    #[tokio::test]
    async fn test_unauthorized_attempt_recording() {
        let acm = AccessControlManager::new();
        acm.record_unauthorized_attempt("attacker", "vault1", "unauthorized access")
            .await
            .unwrap();

        let events = acm.get_security_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, SecurityEventType::UnauthorizedAccessAttempt);
    }

    #[tokio::test]
    async fn test_failed_attempts_tracking() {
        let acm = AccessControlManager::new();

        for _ in 0..3 {
            acm.record_unauthorized_attempt("attacker", "vault1", "attempt")
                .await
                .unwrap();
        }

        assert_eq!(acm.get_failed_attempts("attacker").await, 3);
        assert!(!acm.is_user_blocked("attacker").await);

        for _ in 0..2 {
            acm.record_unauthorized_attempt("attacker", "vault1", "attempt")
                .await
                .unwrap();
        }

        assert!(acm.is_user_blocked("attacker").await);
    }

    #[tokio::test]
    async fn test_suspicious_withdrawal_alert() {
        let acm = AccessControlManager::new();
        // Use amount that's > 10x the average to trigger Critical severity
        // 100_000_000 * 10 = 1_000_000_000, so we need > 1_000_000_000
        acm.record_suspicious_withdrawal("user1", "vault1", 1_000_000_001, 100_000_000)
            .await
            .unwrap();

        let critical_alerts = acm.get_alerts_by_severity(AlertSeverity::Critical).await;
        assert_eq!(critical_alerts.len(), 1);
        assert_eq!(
            critical_alerts[0].event_type,
            SecurityEventType::SuspiciousWithdrawal
        );
    }

    #[tokio::test]
    async fn test_rapid_transaction_detection() {
        let acm = AccessControlManager::new();
        acm.record_rapid_transactions("user1", "vault1", 10, 5)
            .await
            .unwrap();

        let high_alerts = acm.get_alerts_by_severity(AlertSeverity::High).await;
        assert_eq!(high_alerts.len(), 1);
        assert_eq!(
            high_alerts[0].event_type,
            SecurityEventType::RapidTransactionSequence
        );
    }

    #[tokio::test]
    async fn test_clear_failed_attempts() {
        let acm = AccessControlManager::new();
        acm.record_unauthorized_attempt("user1", "vault1", "attempt")
            .await
            .unwrap();

        assert_eq!(acm.get_failed_attempts("user1").await, 1);

        acm.clear_failed_attempts("user1").await.unwrap();
        assert_eq!(acm.get_failed_attempts("user1").await, 0);
    }
}
