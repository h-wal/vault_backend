use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct ReconciliationRow {
    pub id: Uuid,
    pub vault_pda: String,
    pub program_id: String,
    pub network: String,
    pub onchain_balance: i64,
    pub offchain_balance: i64,
    pub discrepancy: i64,
    pub detected_at: DateTime<Utc>,
    pub resolved: bool,
}

pub struct ReconciliationRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ReconciliationRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_discrepancy(&self, entry: &ReconciliationRow) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO reconciliation_logs (
                id,
                vault_pda,
                program_id,
                network,
                onchain_balance,
                offchain_balance,
                discrepancy,
                detected_at,
                resolved
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
            entry.id,
            entry.vault_pda,
            entry.program_id,
            entry.network,
            entry.onchain_balance,
            entry.offchain_balance,
            entry.discrepancy,
            entry.detected_at,
            entry.resolved
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_discrepancy(
        &self,
        id: Uuid,
        vault_pda: &str,
        program_id: &str,
        network: &str,
        onchain_balance: i64,
        offchain_balance: i64,
        discrepancy: i64,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO reconciliation_logs (
                id,
                vault_pda,
                program_id,
                network,
                onchain_balance,
                offchain_balance,
                discrepancy,
                detected_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            "#,
            id,
            vault_pda,
            program_id,
            network,
            onchain_balance,
            offchain_balance,
            discrepancy
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

