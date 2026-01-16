use chrono::NaiveDateTime;
use sqlx::PgPool;

#[derive(Debug)]
pub struct VaultRow {
    pub vault_pda: String,
    pub program_id: String,
    pub network: String,
    pub owner_pubkey: String,
    pub mint: String,
    pub vault_token_account: String,
    pub total_balance: i64,
    pub locked_balance: i64,
    pub available_balance: i64,
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub created_at: NaiveDateTime,
    pub last_synced_at: NaiveDateTime,
}

pub struct VaultRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> VaultRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Upsert a full vault row (low-level helper).
    pub async fn upsert_vault(&self, vault: &VaultRow) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO vaults (
                vault_pda,
                program_id,
                network,
                owner_pubkey,
                mint,
                vault_token_account,
                total_balance,
                locked_balance,
                available_balance,
                total_deposited,
                total_withdrawn,
                created_at,
                last_synced_at
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            ON CONFLICT (vault_pda) DO UPDATE SET
                total_balance = EXCLUDED.total_balance,
                locked_balance = EXCLUDED.locked_balance,
                available_balance = EXCLUDED.available_balance,
                total_deposited = EXCLUDED.total_deposited,
                total_withdrawn = EXCLUDED.total_withdrawn,
                last_synced_at = EXCLUDED.last_synced_at
            "#,
            vault.vault_pda,
            vault.program_id,
            vault.network,
            vault.owner_pubkey,
            vault.mint,
            vault.vault_token_account,
            vault.total_balance,
            vault.locked_balance,
            vault.available_balance,
            vault.total_deposited,
            vault.total_withdrawn,
            vault.created_at,
            vault.last_synced_at,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_vault(&self, vault_pda: &str) -> anyhow::Result<Option<VaultRow>> {
        let row = sqlx::query_as!(
            VaultRow,
            r#"SELECT * FROM vaults WHERE vault_pda = $1"#,
            vault_pda
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(row)
    }

    /// Return all vaults (used by reconciliation worker and analytics).
    pub async fn get_all_vaults(&self) -> anyhow::Result<Vec<VaultRow>> {
        let rows = sqlx::query_as!(
            VaultRow,
            r#"SELECT * FROM vaults ORDER BY created_at ASC"#
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    /// Fetch the vault record for a given owner, if any.
    pub async fn get_vault_by_owner(
        &self,
        owner_pubkey: &str,
    ) -> anyhow::Result<Option<VaultRow>> {
        let row = sqlx::query_as!(
            VaultRow,
            r#"SELECT * FROM vaults WHERE owner_pubkey = $1"#,
            owner_pubkey,
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(row)
    }

    /// Compute total value locked (TVL) across all vaults.
    pub async fn get_tvl(&self) -> anyhow::Result<i64> {
        // Explicitly cast the SUM to BIGINT so SQLx doesn't require the
        // `bigdecimal` feature for NUMERIC.
        let tvl: i64 = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(total_balance)::BIGINT, 0) AS "tvl!: i64" FROM vaults"#,
        )
        .fetch_one(self.pool)
        .await?;

        Ok(tvl)
    }

    /// Insert a new vault when a `VaultInitialized` event is seen.
    ///
    /// Fields we don't get from the event are filled with sensible defaults.
    pub async fn insert_new_vault(
        &self,
        vault_pda: &str,
        owner_pubkey: &str,
        mint: &str,
        timestamp: i64,
    ) -> anyhow::Result<()> {
        // Convert unix timestamp -> NaiveDateTime, fall back to now() if conversion fails.
        use chrono::{DateTime, Utc};
        let created_at = {
            let utc_dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
                .unwrap_or_else(|| Utc::now());
            utc_dt.naive_utc()
        };

        let vault = VaultRow {
            vault_pda: vault_pda.to_string(),
            program_id: "".to_string(), // can be filled with real program id in a later migration
            network: "localnet".to_string(),
            owner_pubkey: owner_pubkey.to_string(),
            mint: mint.to_string(),
            vault_token_account: "".to_string(),
            total_balance: 0,
            locked_balance: 0,
            available_balance: 0,
            total_deposited: 0,
            total_withdrawn: 0,
            created_at,
            last_synced_at: created_at,
        };

        self.upsert_vault(&vault).await
    }

    /// Set balances directly from an on-chain event (e.g. deposit).
    pub async fn set_balance_from_event(
        &self,
        vault_pda: &str,
        new_total_balance: i64,
        timestamp: i64,
    ) -> anyhow::Result<()> {
        use chrono::{DateTime, Utc};
        let utc_dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        let ts = utc_dt.naive_utc();

        sqlx::query!(
            r#"
            UPDATE vaults
            SET
                total_balance     = $2,
                available_balance = $2,
                last_synced_at    = $3
            WHERE vault_pda = $1
            "#,
            vault_pda,
            new_total_balance,
            ts,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Apply a withdraw event to the off-chain balances.
    pub async fn apply_withdraw(&self, vault_pda: &str, amount: i64) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE vaults
            SET
                total_balance     = total_balance - $2,
                available_balance = available_balance - $2,
                total_withdrawn   = total_withdrawn + $2,
                last_synced_at    = now()
            WHERE vault_pda = $1
            "#,
            vault_pda,
            amount,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Apply a lock event: move from available -> locked.
    pub async fn apply_lock(&self, vault_pda: &str, amount: i64) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE vaults
            SET
                available_balance = available_balance - $2,
                locked_balance    = locked_balance + $2,
                last_synced_at    = now()
            WHERE vault_pda = $1
            "#,
            vault_pda,
            amount,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Apply an unlock event: move from locked -> available.
    pub async fn apply_unlock(&self, vault_pda: &str, amount: i64) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE vaults
            SET
                available_balance = available_balance + $2,
                locked_balance    = locked_balance - $2,
                last_synced_at    = now()
            WHERE vault_pda = $1
            "#,
            vault_pda,
            amount,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Apply a transfer between two vaults.
    pub async fn apply_transfer(
        &self,
        from_vault: &str,
        to_vault: &str,
        amount: i64,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // Debit from_vault
        sqlx::query!(
            r#"
            UPDATE vaults
            SET
                total_balance     = total_balance - $2,
                available_balance = available_balance - $2,
                last_synced_at    = now()
            WHERE vault_pda = $1
            "#,
            from_vault,
            amount,
        )
        .execute(&mut *tx)
        .await?;

        // Credit to_vault
        sqlx::query!(
            r#"
            UPDATE vaults
            SET
                total_balance     = total_balance + $2,
                available_balance = available_balance + $2,
                last_synced_at    = now()
            WHERE vault_pda = $1
            "#,
            to_vault,
            amount,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

