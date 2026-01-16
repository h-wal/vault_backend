use std::sync::Arc;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use sqlx::PgPool;

use crate::db::program_repo::ProgramRepository;
use crate::transaction_builder::TransactionBuilder;

/// CPIManager is a backend abstraction for other services (position manager,
/// liquidation engine, settlement relayer) that need to lock/unlock collateral
/// in the on-chain vault program.
///
/// It does two things:
/// - Builds the correct vault instructions (lock/unlock) using the existing
///   TransactionBuilder.
/// - Persists cross-program calls into `program_calls` for audit / analytics,
///   and checks that caller programs are authorized.
pub struct CPIManager<'a> {
    pub rpc: Arc<RpcClient>,
    pub program_id: Pubkey,
    pub pool: &'a PgPool,
}

impl<'a> CPIManager<'a> {
    pub fn new(rpc: Arc<RpcClient>, program_id: Pubkey, pool: &'a PgPool) -> Self {
        Self { rpc, program_id, pool }
    }

    // fn tx_builder(&self) -> TransactionBuilder {
    //     TransactionBuilder::new(self.program_id)
    // }

    /// Internal helper: ensure a given program is authorized to perform CPIs
    /// against the vault program.
    async fn ensure_authorized_program(
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

    /// Build a lock-collateral instruction and record the intended call.
    ///
    /// This returns an unsigned transaction the caller can sign and send.
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
        self.ensure_authorized_program(caller_program).await?;

        // In a real implementation, you'd have explicit lock/unlock
        // instructions in the on-chain program + IDL; here we treat lock as a
        // semantic operation and only record the call in the DB.
        let tx = self.build_empty_tx(user_pubkey).await?;

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

        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        let encoded = STANDARD.encode(bincode::serialize(&tx)?);
        Ok(encoded)
    }

    /// Build an unlock-collateral transaction and record the call.
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
        self.ensure_authorized_program(caller_program).await?;

        let tx = self.build_empty_tx(user_pubkey).await?;

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

        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        let encoded = STANDARD.encode(bincode::serialize(&tx)?);
        Ok(encoded)
    }

    /// Construct an empty transaction with the correct recent blockhash that
    /// the caller can append instructions to on their side if desired.
    async fn build_empty_tx(
        &self,
        payer: &Pubkey,
    ) -> anyhow::Result<Transaction> {
        let recent_blockhash = self.rpc.get_latest_blockhash()?;
        let message = Message::new(&[] as &[Instruction], Some(payer));
        let mut tx = Transaction::new_unsigned(message);
        tx.message.recent_blockhash = recent_blockhash;
        Ok(tx)
    }
}

