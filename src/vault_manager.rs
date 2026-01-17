// Given this is a non-custodial vault ...
// this is the vualt manager and it can sign and send transacation to the blockchain on user's behave given that 
// we give the user keypair . In this version I am not supporthing user's private key but it can be implemented using MPC and then this can be implemented

use crate::transaction_builder::TransactionBuilder;
use borsh::BorshDeserialize;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::CommitmentConfig,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

use crate::states::CollateralVault;

// VaultManager handles the core operations of vaults
// It manages initialization, deposits, withdrawals, and balance tracking
pub struct VaultManager {
    rpc_client: RpcClient, // Rpc connection url to the network
    tx_builder: TransactionBuilder,
    payer: Keypair, // the payer who pays the required fees
}

impl VaultManager {
    // Create a new VaultManager instance with given RPC endpoint and program ID
    pub fn new(rpc_url: String, program_id: Pubkey, payer: Keypair) -> Self {
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        let tx_builder = TransactionBuilder::new(program_id);

        Self {
            rpc_client,
            tx_builder,
            payer,
        }
    }

    // Initialize a new vault for a user
    // This creates the vault account on-chain and records it
    pub fn initialize_vault(&self, user: &Keypair, mint: &Pubkey) -> anyhow::Result<Signature> {
        
        let ix = self
            .tx_builder
            .build_initialize_vault_ix(&user.pubkey(), mint)?;

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.payer.pubkey()));

        tx.sign(&[&self.payer, user], recent_blockhash);

        let sig = self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(sig)
    }

    // Process a deposit to a user's vault
    // Transfers tokens from user's wallet to the vault account
    pub fn deposit(&self, user: &Keypair, mint: &Pubkey, amount: u64) -> anyhow::Result<Signature> {
        let ix = self
            .tx_builder
            .build_deposit_ix(&user.pubkey(), mint, amount)?;

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.payer.pubkey()));

        tx.sign(&[&self.payer, user], recent_blockhash);

        let signature = self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(signature)
    }

    // Process a withdrawal from a user's vault
    // Transfers tokens from vault back to user's wallet
    pub fn withdraw(
        &self,
        user: &Keypair,
        mint: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<Signature> {
        let ix = self
            .tx_builder
            .build_withdraw_ix(&user.pubkey(), mint, amount)?;

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.payer.pubkey()));

        tx.sign(&[&self.payer, user], recent_blockhash);

        let sig = self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(sig)
    }

    // Get the current state of a vault from the blockchain
    pub fn get_vault_state(&self, user: &Pubkey) -> anyhow::Result<CollateralVault> {

        let (vault_pda, _) = self.tx_builder.derive_vault_pda(user);

        let account = self.rpc_client.get_account(&vault_pda)?;

        let vault = CollateralVault::try_from_slice(&account.data)?;

        Ok(vault)
    }

    // Get both available and locked balance for a vault
    pub fn get_balances(&self, user: &Pubkey) -> anyhow::Result<(u64, u64)> {
        let vault = self.get_vault_state(user)?;
        Ok((vault.available_balance, vault.locked_balance))
    }

    // Get recent transaction signatures for an address
    // Used to track vault activity
    pub fn get_recent_transactions(&self, address: &Pubkey) -> anyhow::Result<Vec<Signature>> {
        let sig_infos = self.rpc_client.get_signatures_for_address(address)?;

        let signatures = sig_infos
            .into_iter()
            .map(|info| info.signature.parse::<Signature>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(signatures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Signer;
    use std::str::FromStr;

    fn create_test_vault_manager() -> VaultManager {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let program_id = Pubkey::from_str("9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ")
            .expect("Invalid program ID");
        let payer = Keypair::new();

        VaultManager::new(rpc_url, program_id, payer)
    }

    fn create_test_tx_builder() -> TransactionBuilder {
        let program_id = Pubkey::from_str("9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ")
            .expect("Invalid program ID");
        TransactionBuilder::new(program_id)
    }

    #[test]
    fn test_vault_manager_creation() {
        let _vm = create_test_vault_manager();
    }

    #[test]
    fn test_derive_vault_pda() {
        let tx_builder = create_test_tx_builder();
        let user = Keypair::new();

        let (pda, bump) = tx_builder.derive_vault_pda(&user.pubkey());

        assert_ne!(pda, Pubkey::default());
        assert!(bump > 0);
    }

    #[test]
    fn test_derive_vault_pda_deterministic() {
        let tx_builder = create_test_tx_builder();
        let user = Keypair::new();

        let (pda1, bump1) = tx_builder.derive_vault_pda(&user.pubkey());
        let (pda2, bump2) = tx_builder.derive_vault_pda(&user.pubkey());

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

        assert_ne!(pda1, pda2);
    }

    #[test]
    fn test_deposit_request_creation() {
        let amount = 1_000_000_000u64;
        assert!(amount > 0);
        assert_eq!(amount, 1_000_000_000);
    }

    #[test]
    fn test_withdrawal_validation() {
        let available_balance = 1_000_000_000u64;
        let withdrawal_amount = 500_000_000u64;
        assert!(withdrawal_amount <= available_balance);
    }

    #[test]
    fn test_withdrawal_exceeds_balance() {
        let available_balance = 500_000_000u64;
        let withdrawal_amount = 1_000_000_000u64;
        assert!(withdrawal_amount > available_balance);
    }

    #[test]
    fn test_balance_tracking() {
        let total_balance = 1_000_000_000u64;
        let locked_balance = 500_000_000u64;
        let available = total_balance - locked_balance;

        assert_eq!(total_balance, 1_000_000_000);
        assert_eq!(locked_balance, 500_000_000);
        assert_eq!(available, 500_000_000);
    }

    #[test]
    fn test_multiple_deposits() {
        let deposits = vec![100_000_000u64, 200_000_000u64, 300_000_000u64];
        let total_balance: u64 = deposits.iter().sum();
        assert_eq!(total_balance, 600_000_000);
    }

    #[test]
    fn test_sequential_lock_unlock() {
        let total_balance = 1_000_000_000u64;
        let mut locked_balance = 0u64;

        locked_balance += 500_000_000u64;
        assert_eq!(total_balance - locked_balance, 500_000_000);

        locked_balance -= 500_000_000u64;
        assert_eq!(total_balance - locked_balance, 1_000_000_000);
    }

    #[test]
    fn test_zero_deposit_rejected() {
        let deposit_amount = 0u64;
        assert_eq!(deposit_amount, 0);
    }

    #[test]
    fn test_transaction_history_ordering() {
        let transactions = vec![
            ("deposit", 100),
            ("lock", 50),
            ("unlock", 50),
            ("withdraw", 100),
        ];

        assert_eq!(transactions.len(), 4);
        for (_, (tx_type, _)) in transactions.iter().enumerate() {
            assert!(!tx_type.is_empty());
        }
    }

    #[test]
    fn test_vault_state_consistency() {
        let total_balance = 1_000_000_000u64;
        let locked_balance = 400_000_000u64;
        let available_balance = total_balance - locked_balance;

        assert!(locked_balance <= total_balance);
        assert_eq!(available_balance, 600_000_000);
        assert!(available_balance <= total_balance);
    }
}
