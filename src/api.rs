use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum::response::Response;
use axum::extract::ws::{Message as WsMessage, WebSocket};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use sqlx::PgPool;

use crate::db::{pool::create_pg_pool, transaction_repo::TransactionRepository, vault_repo::VaultRepository};
use crate::transaction_builder::TransactionBuilder;

// App state passed to all route handlers
#[derive(Clone)]
pub struct AppState {
    pub rpc: Arc<RpcClient>,
    pub program_id: Pubkey,
    pub pool: PgPool,
}

impl AppState {
    // Helper to create a transaction builder
    pub fn tx_builder(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.program_id)
    }
}

#[derive(Deserialize)]
pub struct InitializeVaultRequest {
    pub user_pubkey: String,
    pub mint: String,
}

#[derive(Deserialize)]
pub struct DepositRequest {
    pub user_pubkey: String,
    pub mint: String,
    pub amount: u64,
}

#[derive(Deserialize)]
pub struct WithdrawRequest {
    pub user_pubkey: String,
    pub mint: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct BuildTransactionResponse {
    // Base64 encoded transaction that's ready to be signed
    pub transaction: String,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub vault_pda: String,
    pub total_balance: i64,
    pub available_balance: i64,
    pub locked_balance: i64,
}

#[derive(Serialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<TransactionSummary>,
}

#[derive(Serialize)]
pub struct TransactionSummary {
    pub tx_signature: String,
    pub tx_type: String,
    pub amount: i64,
    pub slot: i64,
}

#[derive(Serialize)]
pub struct TvlResponse {
    pub tvl: i64,
}

// Build a transaction and return it unsigned for external signing
async fn build_tx_response(
    rpc: &RpcClient,
    payer: &Pubkey,
    ix: solana_sdk::instruction::Instruction,
) -> anyhow::Result<BuildTransactionResponse> {
    let recent_blockhash = rpc.get_latest_blockhash()?;

    let message = Message::new(&[ix], Some(payer));
    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = recent_blockhash;

    let bytes = bincode::serialize(&tx)?;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let encoded = STANDARD.encode(bytes);

    Ok(BuildTransactionResponse { transaction: encoded })
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/vault/initialize", post(initialize_vault))
        .route("/vault/deposit", post(deposit))
        .route("/vault/withdraw", post(withdraw))
        .route("/vault/balance/{user}", get(get_balance))
        .route("/vault/transactions/{user}", get(get_transactions))
        .route("/vault/tvl", get(get_tvl))
        .route("/ws/vaults", get(ws_vaults))
        .with_state(state)
}

async fn ws_vaults(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    // Very simple implementation: on connect, send current TVL and then
    // periodically push updated TVL. In production you'd back this with a
    // broadcast channel fed by the indexer & reconciliation worker.
    use tokio::time::{sleep, Duration};

    loop {
        let repo = VaultRepository::new(&state.pool);
        match repo.get_tvl().await {
            Ok(tvl) => {
                let msg = serde_json::to_string(&TvlResponse { tvl }).unwrap_or_default();
                if socket.send(WsMessage::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            Err(_) => {
                // Ignore errors, client will see stale data.
            }
        }

        // Throttle updates to avoid spamming clients.
        sleep(Duration::from_secs(5)).await;
    }
}

async fn initialize_vault(
    State(state): State<AppState>,
    Json(body): Json<InitializeVaultRequest>,
) -> impl IntoResponse {
    (|| async {
        let user_pubkey = body
            .user_pubkey
            .parse::<Pubkey>()
            .context("invalid user_pubkey")?;
        let mint = body.mint.parse::<Pubkey>().context("invalid mint")?;

        let ix = state
            .tx_builder()
            .build_initialize_vault_ix(&user_pubkey, &mint)?;

        let resp = build_tx_response(&state.rpc, &user_pubkey, ix).await?;
        Ok::<_, anyhow::Error>(Json(resp))
    })()
    .await
    .map_err(internal_error)
}

async fn deposit(
    State(state): State<AppState>,
    Json(body): Json<DepositRequest>,
) -> impl IntoResponse {
    (|| async {
        let user_pubkey = body
            .user_pubkey
            .parse::<Pubkey>()
            .context("invalid user_pubkey")?;
        let mint = body.mint.parse::<Pubkey>().context("invalid mint")?;

        let ix = state
            .tx_builder()
            .build_deposit_ix(&user_pubkey, &mint, body.amount)?;

        let resp = build_tx_response(&state.rpc, &user_pubkey, ix).await?;
        Ok::<_, anyhow::Error>(Json(resp))
    })()
    .await
    .map_err(internal_error)
}

async fn withdraw(
    State(state): State<AppState>,
    Json(body): Json<WithdrawRequest>,
) -> impl IntoResponse {
    (|| async {
        let user_pubkey = body
            .user_pubkey
            .parse::<Pubkey>()
            .context("invalid user_pubkey")?;
        let mint = body.mint.parse::<Pubkey>().context("invalid mint")?;

        let ix = state
            .tx_builder()
            .build_withdraw_ix(&user_pubkey, &mint, body.amount)?;

        let resp = build_tx_response(&state.rpc, &user_pubkey, ix).await?;
        Ok::<_, anyhow::Error>(Json(resp))
    })()
    .await
    .map_err(internal_error)
}

async fn get_balance(
    State(state): State<AppState>,
    Path(user): Path<String>,
) -> impl IntoResponse {
    (|| async {
        let user_pubkey = user.parse::<Pubkey>().context("invalid user pubkey")?;

        let (vault_pda, _) = state.tx_builder().derive_vault_pda(&user_pubkey);

        let repo = VaultRepository::new(&state.pool);
        if let Some(vault) = repo.get_vault(&vault_pda.to_string()).await? {
            let resp = BalanceResponse {
                vault_pda: vault.vault_pda,
                total_balance: vault.total_balance,
                available_balance: vault.available_balance,
                locked_balance: vault.locked_balance,
            };
            Ok::<_, anyhow::Error>(Json(resp))
        } else {
            Err(anyhow::anyhow!("vault not found"))
        }
    })()
    .await
    .map_err(internal_error)
}

async fn get_transactions(
    State(state): State<AppState>,
    Path(user): Path<String>,
) -> impl IntoResponse {
    (|| async {
        let repo = TransactionRepository::new(&state.pool);
        let rows = repo.get_by_user(&user).await?;

        let txs = rows
            .into_iter()
            .map(|row| TransactionSummary {
                tx_signature: row.tx_signature,
                tx_type: row.tx_type,
                amount: row.amount,
                slot: row.slot,
            })
            .collect();

        Ok::<_, anyhow::Error>(Json(TransactionsResponse { transactions: txs }))
    })()
    .await
    .map_err(internal_error)
}

async fn get_tvl(State(state): State<AppState>) -> impl IntoResponse {
    (|| async {
        let repo = VaultRepository::new(&state.pool);
        let tvl = repo.get_tvl().await?;
        Ok::<_, anyhow::Error>(Json(TvlResponse { tvl }))
    })()
    .await
    .map_err(internal_error)
}

fn internal_error(err: anyhow::Error) -> (StatusCode, String) {
    // In a production system you'd log this with `tracing` and return a structured body.
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

/// Helper to bootstrap the server from `main`.
pub async fn run_server() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8899".to_string());
    let program_id = std::env::var("PROGRAM_ID")
        .context("PROGRAM_ID env var not set")?
        .parse::<Pubkey>()
        .context("invalid PROGRAM_ID")?;
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL env var not set")?;

    let rpc = Arc::new(RpcClient::new(rpc_url));
    let pool = create_pg_pool(&database_url).await?;

    let state = AppState {
        rpc,
        program_id,
        pool,
    };

    let app = router(state);

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        .await
        .context("server error")
}

