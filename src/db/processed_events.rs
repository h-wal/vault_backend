use sqlx::PgPool;

/// Struct wrapper used by the indexer; internally just calls the free
/// functions below.
pub struct ProcessedEventsRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> ProcessedEventsRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn is_processed(&self, sig: &str) -> anyhow::Result<bool> {
        is_processed(self.pool, sig).await
    }

    pub async fn mark_processed(&self, sig: &str) -> anyhow::Result<()> {
        mark_processed(self.pool, sig).await
    }
}

pub async fn is_processed(pool: &PgPool, sig: &str) -> anyhow::Result<bool> {
    let exists = sqlx::query!(
        "SELECT 1 FROM processed_events WHERE tx_signature = $1",
        sig
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    Ok(exists)
}

pub async fn mark_processed(pool: &PgPool, sig: &str) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO processed_events (tx_signature) VALUES ($1) ON CONFLICT DO NOTHING",
        sig
    )
    .execute(pool)
    .await?;

    Ok(())
}

