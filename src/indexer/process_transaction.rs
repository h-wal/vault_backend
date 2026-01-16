use sqlx::PgPool;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;

use crate::db::{
    processed_events::ProcessedEventsRepo,
    snapshot_repo::SnapshotRepository,
    transaction_repo::TransactionRepository,
    vault_repo::VaultRepository,
};
use crate::indexer::event_decoder::{decode_events, VaultEvent};
use crate::transaction_builder::TransactionBuilder;

pub async fn process_transaction(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
    signature: &str,
    pool: &PgPool,
    program_id: &solana_sdk::pubkey::Pubkey,
) -> anyhow::Result<()> {
    let processed_repo = ProcessedEventsRepo::new(pool);

    if processed_repo.is_processed(&signature).await? {
        return Ok(()); // already indexed
    }

    let events = decode_events(&tx.transaction)?;

    let tx_repo = TransactionRepository::new(pool);
    let vault_repo = VaultRepository::new(pool);
    let snapshot_repo = SnapshotRepository::new(pool);

    let tx_builder = TransactionBuilder::new(*program_id);

    let slot = tx.slot as i64;
    let block_time = tx.block_time.unwrap_or(0);

    for event in events {
        match event {
            VaultEvent::VaultInitialized {
                vault,
                owner,
                mint,
                timestamp,
            } => {
                vault_repo
                    .insert_new_vault(&vault, &owner, &mint, timestamp)
                    .await?;
            }

            VaultEvent::Deposit {
                user,
                amount,
                new_balance,
                timestamp,
            } => {
                let (vault_pda, _) = tx_builder.derive_vault_pda(&user.parse()?);

                tx_repo
                    .insert_simple(
                        &vault_pda.to_string(),
                        Some(&user),
                        &signature,
                        "deposit",
                        amount as i64,
                        slot,
                        block_time,
                    )
                    .await?;

                vault_repo
                    .set_balance_from_event(
                        &vault_pda.to_string(),
                        new_balance as i64,
                        timestamp,
                    )
                    .await?;
            }

            VaultEvent::Withdraw {
                vault,
                user,
                amount,
            } => {
                tx_repo
                    .insert_simple(
                        &vault,
                        Some(&user),
                        &signature,
                        "withdraw",
                        amount as i64,
                        slot,
                        block_time,
                    )
                    .await?;

                vault_repo
                    .apply_withdraw(&vault, amount as i64)
                    .await?;
            }

            VaultEvent::Lock { vault, amount } => {
                vault_repo.apply_lock(&vault, amount as i64).await?;
            }

            VaultEvent::Unlock { vault, amount } => {
                vault_repo.apply_unlock(&vault, amount as i64).await?;
            }

            VaultEvent::Transfer { from, to, amount } => {
                vault_repo
                    .apply_transfer(&from, &to, amount as i64)
                    .await?;
            }

            VaultEvent::ProgramAuthorized { .. } => {
                // Optional: persist for analytics / audit
            }

            VaultEvent::VaultAuthorityInitialized { .. } => {
                // Optional: persist authority metadata
            }
        }
    }

    // Simple snapshotting strategy: snapshot all vaults at this transaction's time.
    // In a real system you might throttle this (e.g. hourly).
    if let Some(block_time) = tx.block_time {
        use chrono::{DateTime, Utc};
        use crate::db::vault_repo::VaultRow;

        let ts = {
            let utc_dt = DateTime::<Utc>::from_timestamp(block_time, 0)
                .unwrap_or_else(|| Utc::now());
            utc_dt.naive_utc()
        };

        let all_vaults: Vec<VaultRow> = vault_repo.get_all_vaults().await?;
        snapshot_repo
            .snapshot_all_vaults(&all_vaults, ts)
            .await?;
    }

    processed_repo.mark_processed(&signature).await?;

    Ok(())
}

