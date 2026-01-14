use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcTransactionConfig,
};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;
use sqlx::PgPool;

use crate::indexer::process_transaction::process_transaction;

pub struct VaultIndexer {
    rpc: RpcClient,
    pool: PgPool,
    program_id: Pubkey,
}

impl VaultIndexer {
    pub fn new(rpc: RpcClient, pool: PgPool, program_id: Pubkey) -> Self {
        Self {
            rpc,
            pool,
            program_id,
        }
    }

    pub async fn run_once(&self) -> anyhow::Result<()> {
        let signatures = self
            .rpc
            .get_signatures_for_address(&self.program_id)?;

        for sig_info in signatures {
            let signature = sig_info.signature.clone();

            let tx = self.rpc.get_transaction(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    commitment: None,
                    max_supported_transaction_version: None,
                },
            )?;

            // All logic (including idempotency) is handled here
            process_transaction(
                &tx,
                &self.pool,
                &self.program_id,
            )
            .await?;
        }

        Ok(())
    }
}
