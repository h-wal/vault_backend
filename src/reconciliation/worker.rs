use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;
use uuid::Uuid;
use std::str::FromStr;

use crate::db::{
    reconciliation_repo::ReconciliationRepository,
    vault_repo::VaultRepository,
};
use crate::reconciliation::onchain::fetch_token_balance;

pub struct ReconciliationWorker {
    rpc: RpcClient,
    pool: PgPool,
    program_id: Pubkey,
}

impl ReconciliationWorker {
    pub fn new(rpc: RpcClient, pool: PgPool, program_id: Pubkey) -> Self {
        Self {
            rpc,
            pool,
            program_id,
        }
    }

    pub async fn run_once(&self) -> anyhow::Result<()> {
        let vault_repo = VaultRepository::new(&self.pool);
        let reconciliation_repo = ReconciliationRepository::new(&self.pool);

        let vaults = vault_repo.get_all_vaults().await?;

        for vault in vaults {
            let token_account =
                Pubkey::from_str(&vault.vault_token_account)?;

            let onchain_balance =
                fetch_token_balance(&self.rpc, &token_account)?;

            let offchain_balance = vault.total_balance;

            if onchain_balance as i64 != offchain_balance {
                reconciliation_repo
                    .insert_discrepancy(
                        Uuid::new_v4(),
                        &vault.vault_pda,
                        &vault.program_id,
                        &vault.network,
                        onchain_balance as i64,
                        offchain_balance as i64,
                        offchain_balance as i64 - onchain_balance as i64,
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

