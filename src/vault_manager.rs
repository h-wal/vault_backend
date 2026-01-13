use crate::transaction_builder::TransactionBuilder;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

pub struct VaultManager {
    rpc_client: RpcClient,
    tx_builder: TransactionBuilder,
    payer: Keypair,
}

impl VaultManager {
    pub fn new(rpc_url: String, program_id: Pubkey, payer: Keypair) -> Self {
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        let tx_builder = TransactionBuilder::new(program_id);

        Self {
            rpc_client,
            tx_builder,
            payer,
        }
    }

    pub fn deposit(&self, user: &Keypair, mint: &Pubkey, amount: u64) -> anyhow::Result<Signature> {
        let ix = self
            .tx_builder
            .build_deposit_ix(&user.pubkey(), mint, amount)?;

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.payer.pubkey()));

        tx.sign(&[&self.payer, user], recent_blockhash);

        let signature = self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(signature)
    }
}
