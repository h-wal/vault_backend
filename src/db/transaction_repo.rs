use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct TransactionRow {
    pub id: Uuid,
    pub vault_pda: String,
    pub program_id: String,
    pub network: String,
    pub user_pubkey: Option<String>,
    pub tx_signature: String,
    pub tx_type: String,
    pub amount: i64,
    pub slot: i64,
    pub block_time: DateTime<Utc>,
}

pub struct TransactionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> TransactionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_transaction(&self, tx: &TransactionRow) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                id,
                vault_pda,
                program_id,
                network,
                user_pubkey,
                tx_signature,
                tx_type,
                amount,
                slot,
                block_time
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (tx_signature) DO NOTHING
            "#,
            tx.id,
            tx.vault_pda,
            tx.program_id,
            tx.network,
            tx.user_pubkey,
            tx.tx_signature,
            tx.tx_type,
            tx.amount,
            tx.slot,
            tx.block_time
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Convenience helper used by the indexer to persist a transaction.
    pub async fn insert_simple(
        &self,
        vault_pda: &str,
        user_pubkey: Option<&str>,
        tx_signature: &str,
        tx_type: &str,
        amount: i64,
        slot: i64,
        block_time: i64,
    ) -> anyhow::Result<()> {
        let row = TransactionRow {
            id: Uuid::new_v4(),
            vault_pda: vault_pda.to_string(),
            program_id: "".to_string(),
            network: "localnet".to_string(),
            user_pubkey: user_pubkey.map(|s| s.to_string()),
            tx_signature: tx_signature.to_string(),
            tx_type: tx_type.to_string(),
            amount,
            slot,
            // Interpret `block_time` as unix timestamp seconds.
            block_time: DateTime::<Utc>::from_timestamp(block_time, 0)
                .unwrap_or_else(|| Utc::now()),
        };

        self.insert_transaction(&row).await
    }
}

