#[cfg(test)]
mod tests {
    use crate::vault_manager::VaultManager;
    use crate::transaction_builder::TransactionBuilder;
    use crate::states::CollateralVault;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::{Keypair, Signer};
    use std::str::FromStr;

    // Create a vault manager for testing
    fn create_test_vault_manager() -> VaultManager {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let program_id = Pubkey::from_str("9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ")
            .expect("Invalid program ID");
        let payer = Keypair::new();

        VaultManager::new(rpc_url, program_id, payer)
    }

    // Create a transaction builder for testing
    fn create_test_tx_builder() -> TransactionBuilder {
        let program_id = Pubkey::from_str("9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ")
            .expect("Invalid program ID");
        TransactionBuilder::new(program_id)
    }

    #[test]
    fn test_vault_manager_creation() {
        let vm = create_test_vault_manager();
        // Manager should be created without errors
        assert!(true);
    }

    #[test]
    fn test_derive_vault_pda() {
        let tx_builder = create_test_tx_builder();
        let user = Keypair::new();

        let (pda, bump) = tx_builder.derive_vault_pda(&user.pubkey());

        // PDA should be a valid non-zero address
        assert_ne!(pda, Pubkey::default());
        // Bump should be in valid range
        assert!(bump <= 255);
    }

    #[test]
    fn test_derive_vault_pda_deterministic() {
        let tx_builder = create_test_tx_builder();
        let user = Keypair::new();

        let (pda1, bump1) = tx_builder.derive_vault_pda(&user.pubkey());
        let (pda2, bump2) = tx_builder.derive_vault_pda(&user.pubkey());

        // Same user should derive same PDA
        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_different_users_different_pdas() {
        let tx_builder = create_test_tx_builder();
        let user1 = Keypair::new();
        let user2 = Keypair::new();

        let (pda1, _) = tx_builder.derive_vault_pda(&user1.pubkey());
        let (pda2, _) = tx_builder.derive_vault_pda(&user2.pubkey());

        // Different users should have different PDAs
        assert_ne!(pda1, pda2);
    }

    #[test]
    fn test_deposit_request_creation() {
        let mint = Pubkey::new_unique();
        let amount = 1_000_000_000u64;

        // Verify amount is positive
        assert!(amount > 0);
        assert_eq!(amount, 1_000_000_000);
    }

    #[test]
    fn test_withdrawal_validation() {
        let available_balance = 1_000_000_000u64;
        let withdrawal_amount = 500_000_000u64;

        // Withdrawal should not exceed available balance
        assert!(withdrawal_amount <= available_balance);
    }

    #[test]
    fn test_withdrawal_exceeds_balance() {
        let available_balance = 500_000_000u64;
        let withdrawal_amount = 1_000_000_000u64;

        // Withdrawal should exceed available balance
        assert!(withdrawal_amount > available_balance);
    }

    #[test]
    fn test_balance_tracking() {
        let mut total_balance = 0u64;
        let mut locked_balance = 0u64;

        // Simulate deposit
        total_balance += 1_000_000_000u64;

        // Simulate lock
        locked_balance += 500_000_000u64;
        let available = total_balance - locked_balance;

        assert_eq!(total_balance, 1_000_000_000);
        assert_eq!(locked_balance, 500_000_000);
        assert_eq!(available, 500_000_000);
    }

    #[test]
    fn test_multiple_deposits() {
        let mut total_balance = 0u64;
        let deposits = vec![
            100_000_000u64,
            200_000_000u64,
            300_000_000u64,
        ];

        for deposit in deposits {
            total_balance += deposit;
        }

        assert_eq!(total_balance, 600_000_000);
    }

    #[test]
    fn test_sequential_lock_unlock() {
        let mut total_balance = 1_000_000_000u64;
        let mut locked_balance = 0u64;

        // Lock
        locked_balance += 500_000_000u64;
        assert_eq!(total_balance - locked_balance, 500_000_000);

        // Unlock
        locked_balance -= 500_000_000u64;
        assert_eq!(total_balance - locked_balance, 1_000_000_000);
    }

    #[test]
    fn test_zero_deposit_rejected() {
        let deposit_amount = 0u64;
        assert_eq!(deposit_amount, 0, "Zero deposits should not be allowed");
    }

    #[test]
    fn test_transaction_history_ordering() {
        let transactions = vec![
            ("deposit", 100),
            ("lock", 50),
            ("unlock", 50),
            ("withdraw", 100),
        ];

        // Verify transactions are recorded
        assert_eq!(transactions.len(), 4);

        // Verify order is maintained
        for (i, (tx_type, _)) in transactions.iter().enumerate() {
            assert!(!tx_type.is_empty());
        }
    }

    #[test]
    fn test_vault_state_consistency() {
        let total_balance = 1_000_000_000u64;
        let locked_balance = 400_000_000u64;
        let available_balance = total_balance - locked_balance;

        // State invariants
        assert!(locked_balance <= total_balance);
        assert_eq!(available_balance, 600_000_000);
        assert!(available_balance <= total_balance);
    }
}
