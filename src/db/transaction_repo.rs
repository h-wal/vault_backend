use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::NaiveDateTime;

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
    pub block_time: NaiveDateTime,
}

pub struct TransactionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> TransactionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_transaction(&self, tx: &TransactionRow) -> anyhow::Result<()> {
        sqlx::query(
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
            VALUES ($1,$2,$3,$4,$5,$6,$7::transaction_type,$8,$9,$10)
            ON CONFLICT (tx_signature) DO NOTHING
            "#,
        )
        .bind(tx.id)
        .bind(&tx.vault_pda)
        .bind(&tx.program_id)
        .bind(&tx.network)
        .bind(&tx.user_pubkey)
        .bind(&tx.tx_signature)
        .bind(&tx.tx_type)
        .bind(tx.amount)
        .bind(tx.slot)
        .bind(tx.block_time)
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
            block_time: {
                use chrono::{DateTime, Utc};
                let utc_dt = DateTime::<Utc>::from_timestamp(block_time, 0)
                    .unwrap_or_else(|| Utc::now());
                utc_dt.naive_utc()
            },
        };

        self.insert_transaction(&row).await
    }

    /// Fetch all transactions for a given user public key.
    pub async fn get_by_user(
        &self,
        user_pubkey: &str,
    ) -> anyhow::Result<Vec<TransactionRow>> {
        let rows = sqlx::query(
            r#"
            SELECT
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
            FROM transactions
            WHERE user_pubkey = $1
            ORDER BY slot DESC
            "#,
        )
        .bind(user_pubkey)
        .fetch_all(self.pool)
        .await?;

        let rows = rows
            .into_iter()
            .map(|row: sqlx::postgres::PgRow| TransactionRow {
                id: row.get("id"),
                vault_pda: row.get("vault_pda"),
                program_id: row.get("program_id"),
                network: row.get("network"),
                user_pubkey: row.get("user_pubkey"),
                tx_signature: row.get("tx_signature"),
                tx_type: row.get("tx_type"),
                amount: row.get("amount"),
                slot: row.get("slot"),
                block_time: row.get("block_time"),
            })
            .collect();

        Ok(rows)
    }
}

