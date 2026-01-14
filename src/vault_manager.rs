use crate::transaction_builder::TransactionBuilder;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

use crate::states::CollateralVault;

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

    pub fn initialize_vault(&self, user: &Keypair, mint: &Pubkey) -> anyhow::Result<Signature> {
        let ix = self
            .tx_builder
            .build_initialize_vault_ix(&user.pubkey(), mint)?;

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.payer.pubkey()));

        tx.sign(&[&self.payer, user], recent_blockhash);

        let sig = self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(sig)
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

    pub fn withdraw(
        &self,
        user: &Keypair,
        mint: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<Signature> {
        let ix = self
            .tx_builder
            .build_withdraw_ix(&user.pubkey(), mint, amount)?;

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let mut tx = Transaction::new_with_payer(&[ix], Some(&self.payer.pubkey()));

        tx.sign(&[&self.payer, user], recent_blockhash);

        let sig = self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(sig)
    }

    pub fn get_vault_state(&self, user: &Pubkey) -> anyhow::Result<CollateralVault> {

        let (vault_pda, _) = self.tx_builder.derive_vault_pda(user);

        let account = self.rpc_client.get_account(&vault_pda)?;

        let vault = CollateralVault::try_from_slice(&account.data)?;

        Ok(vault)
    }

    pub fn get_balances(&self, user: &Pubkey) -> anyhow::Result<(u64, u64)> {
        let vault = self.get_vault_state(user)?;
        Ok((vault.available_balance, vault.locked_balance))
    }

    pub fn get_recent_transactions(&self, address: &Pubkey) -> anyhow::Result<Vec<Signature>> {
        let sig_infos = self.rpc_client.get_signatures_for_address(address)?;

        let signatures = sig_infos
            .into_iter()
            .map(|info| info.signature.parse::<Signature>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(signatures)
    }
}
