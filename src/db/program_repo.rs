use chrono::NaiveDateTime;
use sqlx::PgPool;

#[derive(Debug)]
pub struct AuthorizedProgramRow {
    pub program_id: String,
    pub admin_pubkey: String,
    pub added_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct ProgramCallRow {
    pub tx_signature: String,
    pub caller_program: String,
    pub vault_pda: String,
    pub instruction: String,
    pub amount: Option<i64>,
    pub slot: i64,
    pub block_time: NaiveDateTime,
}

pub struct ProgramRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ProgramRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn is_program_authorized(
        &self,
        program_id: &str,
    ) -> anyhow::Result<bool> {
        let exists = sqlx::query!(
            r#"
            SELECT 1 AS "exists!"
            FROM authorized_programs
            WHERE program_id = $1
            "#,
            program_id,
        )
        .fetch_optional(self.pool)
        .await?
        .is_some();

        Ok(exists)
    }

    pub async fn insert_authorized_program(
        &self,
        program_id: &str,
        admin_pubkey: &str,
        added_at: NaiveDateTime,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO authorized_programs (
                program_id,
                admin_pubkey,
                added_at
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (program_id) DO NOTHING
            "#,
            program_id,
            admin_pubkey,
            added_at,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_program_call(
        &self,
        tx_signature: &str,
        caller_program: &str,
        vault_pda: &str,
        instruction: &str,
        amount: Option<i64>,
        slot: i64,
        block_time: NaiveDateTime,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO program_calls (
                tx_signature,
                caller_program,
                vault_pda,
                instruction,
                amount,
                slot,
                block_time
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (tx_signature) DO NOTHING
            "#,
            tx_signature,
            caller_program,
            vault_pda,
            instruction,
            amount,
            slot,
            block_time,
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

