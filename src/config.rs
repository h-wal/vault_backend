use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use std::env;

pub struct Config {
    pub rpc_url: String,
    pub program_id: Pubkey,
    pub database_url: String,
    pub server_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let rpc_url = env::var("RPC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8899".to_string());

        let program_id = env::var("PROGRAM_ID")
            .context("PROGRAM_ID environment variable not set")?
            .parse::<Pubkey>()
            .context("Invalid PROGRAM_ID format")?;

        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable not set")?;

        let server_addr = env::var("SERVER_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        Ok(Self {
            rpc_url,
            program_id,
            database_url,
            server_addr,
        })
    }
}
