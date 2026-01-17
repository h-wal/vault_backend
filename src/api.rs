use std::net::SocketAddr; // here we import the SocketAddr struct this includes the network address and port number
use std::sync::Arc; // here we import the arc struct (the shared state between multiple threads)

use anyhow::Context;
use axum::{ // we are using the axum framework for the web server
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

use crate::config::Config;
use crate::db::{pool::create_pg_pool, transaction_repo::TransactionRepository, vault_repo::VaultRepository};
use crate::transaction_builder::TransactionBuilder;

#[derive(Clone)]
pub struct AppState { // this is the state of the application (this includes the rpc client, the program id, and the database pool)
    pub rpc: Arc<RpcClient>, // this is the rpc client (this is used to interact with the solana blockchain)
    pub program_id: Pubkey, // this is the program id (this is used to identify the program)
    pub pool: PgPool, // this is the database pool (this is used to interact with the database)
}

impl AppState { // this is the implementation of the app state (this includes the transaction builder)

    pub fn tx_builder(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.program_id)
    }

}

#[derive(Deserialize)]
pub struct InitializeVaultRequest { // this is the request body for the initialize vault endpoint
    pub user_pubkey: String, // this is the user pubkey (this is used to identify the user)
    pub mint: String, // this is the mint (this is used to identify the mint)
}

#[derive(Deserialize)]
pub struct DepositRequest { // this is the request body for the deposit endpoint
    pub user_pubkey: String,
    pub mint: String,
    pub amount: u64, // the amount to be deposited 
}

#[derive(Deserialize)]
pub struct WithdrawRequest { // this is the request body for the withdraw endpoint
    pub user_pubkey: String,
    pub mint: String,
    pub amount: u64, // amount to be withdrawn
}

#[derive(Serialize)]
pub struct BuildTransactionResponse { // this is the response body for the build transaction endpoint
    pub transaction: String, // this is the transaction (this is the transaction which will be signed by the user)
}

#[derive(Serialize)]
pub struct BalanceResponse { // this is the response body for the balance endpoint
    pub vault_pda: String, // this is a program derived address (PDA) which is used to identify the vault where the users balance is stored derived from the user's pubkey as one of the seeds
    pub total_balance: i64, // this is the total balance of the vault including locked + available balance
    pub available_balance: i64, // this is the available balance (this is the balance that can be withdrawn )
    pub locked_balance: i64, // this is the locked balance (cannot be withdrawn)
}

#[derive(Serialize)]
pub struct TransactionsResponse { // this is the response body for the transactions endpoint
    pub transactions: Vec<TransactionSummary>,
}

#[derive(Serialize)]
pub struct TransactionSummary { // this is the response body for the transaction summary endpoint
    pub tx_signature: String, // tx signature of the transaction
    pub tx_type: String, // type of the transaction
    pub amount: i64, // amount of the transaction
    pub slot: i64, // slot of the transaction 
}

#[derive(Serialize)]
pub struct TvlResponse { // this is the response body for the tvl endpoint
    pub tvl: i64, // total value locked (TVL) of all vaults (this is the total value of all the vaults in the database)
}

async fn build_tx_response( // this is the function to build the transaction response and return it unsigned for external signing
    rpc: &RpcClient,
    payer: &Pubkey,
    ix: solana_sdk::instruction::Instruction, // this is the instruction to be executed
) -> anyhow::Result<BuildTransactionResponse> {
    let recent_blockhash = rpc.get_latest_blockhash()?; // getting the latest blockhash from the rpc client

    let message = Message::new(&[ix], Some(payer)); // creating a new message with the instruction and the payer
    let mut tx = Transaction::new_unsigned(message); // creating a new transaction with the message
    tx.message.recent_blockhash = recent_blockhash; // setting the recent blockhash to the recent blockhash

    let bytes = bincode::serialize(&tx)?; // serializing the transaction
    use base64::engine::general_purpose::STANDARD; // using the standard base64 engine  
    use base64::Engine; // using the base64 engine
    let encoded = STANDARD.encode(bytes); // encoding the transaction   

    Ok(BuildTransactionResponse { transaction: encoded }) // returning the transaction response         
}

pub fn router(state: AppState) -> Router { // this is the router for the api
    Router::new()
        .route("/vault/initialize", post(initialize_vault))
        .route("/vault/deposit", post(deposit))
        .route("/vault/withdraw", post(withdraw))
        .route("/vault/balance/{user}", get(get_balance))
        .route("/vault/transactions/{user}", get(get_transactions))
        .route("/vault/tvl", get(get_tvl))
        .route("/ws/vaults", get(ws_vaults))
        .with_state(state) // passing the state to the router  
}

async fn ws_vaults( // this is the websocket endpoint for the api
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    
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

pub async fn run_server() -> anyhow::Result<()> {

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;

    let rpc = Arc::new(RpcClient::new(config.rpc_url));
    let pool = create_pg_pool(&config.database_url).await?;

    let state = AppState {
        rpc,
        program_id: config.program_id,
        pool,
    };

    let app = router(state);

    let addr: SocketAddr = config.server_addr.parse()?;
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        .await
        .context("server error")
        
}

