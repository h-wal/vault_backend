use std::sync::Arc;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use sqlx::PgPool;

use crate::db::program_repo::ProgramRepository;
use crate::transaction_builder::TransactionBuilder;

/// CPIManager is the  abstraction layer  for other services (position manager,
/// liquidation engine, settlement relayer) it locks/unlocks collateral
/// in the on-chain vault program.

pub struct CPIManager<'a> {
    pub rpc: Arc<RpcClient>,
    pub program_id: Pubkey,
    pub pool: &'a PgPool,
    pub payer: Option<Keypair>,
}

impl<'a> CPIManager<'a> {

    pub fn new(rpc: Arc<RpcClient>, program_id: Pubkey, pool: &'a PgPool) -> Self {
        Self { rpc, program_id, pool, payer: None }
    }

    /// Create CPIManager with a payer keypair for sending transactions
    pub fn new_with_payer(rpc: Arc<RpcClient>, program_id: Pubkey, pool: &'a PgPool, payer: Keypair) -> Self {
        Self { rpc, program_id, pool, payer: Some(payer) }
    }

    fn tx_builder(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.program_id)
    }

    async fn ensure_authorized_program( //checks authority of the pubkey willign to make the cpi
        &self,
        program_id: &Pubkey,
    ) -> anyhow::Result<()> {
        let repo = ProgramRepository::new(self.pool);
        let is_authorized = repo
            .is_program_authorized(&program_id.to_string())
            .await?;

        if !is_authorized {
            anyhow::bail!("unauthorized CPI caller: {}", program_id);
        }

        Ok(())
    }


    pub async fn build_lock_collateral_tx(
        &self,
        caller_program: &Pubkey,
        vault_pda: &Pubkey,
        user_pubkey: &Pubkey,
        _mint: &Pubkey,
        amount: u64,
        slot: i64,
        block_time: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<String> {
        // Verify the caller program is authorized to make CPI calls
        self.ensure_authorized_program(caller_program).await?;

        // Build the actual lock_collateral instruction
        let tx_builder = self.tx_builder();
        let lock_ix = tx_builder.build_lock_collateral_ix(caller_program, user_pubkey, amount)?;

        // Build a transaction with the lock instruction
        let recent_blockhash = self.rpc.get_latest_blockhash()?;
        let message = Message::new(&[lock_ix], Some(user_pubkey));
        let mut tx = Transaction::new_unsigned(message);
        tx.message.recent_blockhash = recent_blockhash;

        // Record the call in the database for audit trail
        let repo = ProgramRepository::new(self.pool);
        repo
            .insert_program_call(
                &tx.signatures
                    .get(0)
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                &caller_program.to_string(),
                &vault_pda.to_string(),
                "lock",
                Some(amount as i64),
                slot,
                block_time.naive_utc(),
            )
            .await?;

        // Serialize and base64 encode the transaction
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        let encoded = STANDARD.encode(bincode::serialize(&tx)?);
        Ok(encoded)
    }

    /// Build an unlock-collateral transaction with actual on-chain instruction
    /// This unlocks locked balance back to available balance on the blockchain
    pub async fn build_unlock_collateral_tx(
        &self,
        caller_program: &Pubkey,
        vault_pda: &Pubkey,
        user_pubkey: &Pubkey,
        _mint: &Pubkey,
        amount: u64,
        slot: i64,
        block_time: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<String> {
        // Verify the caller program is authorized to make CPI calls
        self.ensure_authorized_program(caller_program).await?;

        // Build the actual unlock_collateral instruction
        let tx_builder = self.tx_builder();
        let unlock_ix = tx_builder.build_unlock_collateral_ix(caller_program, user_pubkey, amount)?;

        // Build a transaction with the unlock instruction
        let recent_blockhash = self.rpc.get_latest_blockhash()?;
        let message = Message::new(&[unlock_ix], Some(user_pubkey));
        let mut tx = Transaction::new_unsigned(message);
        tx.message.recent_blockhash = recent_blockhash;

        // Record the call in the database for audit trail
        let repo = ProgramRepository::new(self.pool);
        repo
            .insert_program_call(
                &tx.signatures
                    .get(0)
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                &caller_program.to_string(),
                &vault_pda.to_string(),
                "unlock",
                Some(amount as i64),
                slot,
                block_time.naive_utc(),
            )
            .await?;

        // Serialize and base64 encode the transaction
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        let encoded = STANDARD.encode(bincode::serialize(&tx)?);
        Ok(encoded)
    }

    /// Lock collateral and send the transaction to the blockchain
    /// This method requires a payer to be set via `new_with_payer`
    pub async fn lock_collateral(
        &self,
        caller_program: &Pubkey,
        user_pubkey: &Pubkey,
        amount: u64,
        slot: i64,
        block_time: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Signature> {
        let payer = self.payer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CPIManager must be created with payer to send transactions"))?;

        // Verify authorization
        self.ensure_authorized_program(caller_program).await?;

        // Build the lock instruction
        let tx_builder = self.tx_builder();
        let lock_ix = tx_builder.build_lock_collateral_ix(caller_program, user_pubkey, amount)?;

        // Build and send transaction
        let recent_blockhash = self.rpc.get_latest_blockhash()?;
        let mut tx = Transaction::new_with_payer(&[lock_ix], Some(&payer.pubkey()));
        tx.sign(&[payer], recent_blockhash);

        let signature = self.rpc.send_and_confirm_transaction(&tx)?;

        // Record in database for audit trail
        let (vault_pda, _) = tx_builder.derive_vault_pda(user_pubkey);
        let repo = ProgramRepository::new(self.pool);
        repo
            .insert_program_call(
                &signature.to_string(),
                &caller_program.to_string(),
                &vault_pda.to_string(),
                "lock",
                Some(amount as i64),
                slot,
                block_time.naive_utc(),
            )
            .await?;

        Ok(signature)
    }

    /// Unlock collateral and send the transaction to the blockchain
    /// This method requires a payer to be set via `new_with_payer`
    pub async fn unlock_collateral(
        &self,
        caller_program: &Pubkey,
        user_pubkey: &Pubkey,
        amount: u64,
        slot: i64,
        block_time: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Signature> {
        let payer = self.payer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CPIManager must be created with payer to send transactions"))?;

        // Verify authorization
        self.ensure_authorized_program(caller_program).await?;

        // Build the unlock instruction
        let tx_builder = self.tx_builder();
        let unlock_ix = tx_builder.build_unlock_collateral_ix(caller_program, user_pubkey, amount)?;

        // Build and send transaction
        let recent_blockhash = self.rpc.get_latest_blockhash()?;
        let mut tx = Transaction::new_with_payer(&[unlock_ix], Some(&payer.pubkey()));
        tx.sign(&[payer], recent_blockhash);

        let signature = self.rpc.send_and_confirm_transaction(&tx)?;

        // Record in database for audit trail
        let (vault_pda, _) = tx_builder.derive_vault_pda(user_pubkey);
        let repo = ProgramRepository::new(self.pool);
        repo
            .insert_program_call(
                &signature.to_string(),
                &caller_program.to_string(),
                &vault_pda.to_string(),
                "unlock",
                Some(amount as i64),
                slot,
                block_time.naive_utc(),
            )
            .await?;

        Ok(signature)
    }

    
}

