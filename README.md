# Vault Backend (Solana Collateral Vault Service)

Rust backend that manages collateral vault operations on Solana and maintains an off-chain view of state in PostgreSQL. It exposes a REST API + WebSocket stream for vault lifecycle operations (initialize, deposit, withdraw, balance/history, TVL) and runs indexing/reconciliation to keep the database consistent with on-chain state.

## Documentation

- [SUMMARY.md](SUMMARY.md): implementation overview + statistics
- [API_DOCUMENTATION.md](API_DOCUMENTATION.md): REST + WebSocket specs
- [TESTING_GUIDE.md](TESTING_GUIDE.md): how to run unit tests and manual/API testing
- [VERIFICATION_REPORT.md](VERIFICATION_REPORT.md): verification checklist and results

## Key Components

- `src/bin/server.rs`: HTTP server entrypoint
- `src/api.rs`: REST + WebSocket routes/handlers
- `src/vault_manager.rs`: vault lifecycle operations
- `src/transaction_builder.rs`: builds Solana transactions/instructions
- `src/cpi_manager.rs`: cross-program invocation helpers
- `src/indexer/`: transaction/event processing
- `src/reconciliation/`: on-chain vs DB reconciliation worker
- `src/db/`: Postgres repositories and queries

## Tech Stack

- Rust 2021, Tokio, Axum
- PostgreSQL via `sqlx`
- Solana SDK / RPC client
- Tracing for structured logging

## Quick Start

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Run
```bash
export RUST_LOG=info,vault_backend=debug
export DATABASE_URL=postgresql://user:pass@localhost:5432/vault_db
export SOLANA_RPC_URL=http://localhost:8899

cargo run --bin server
```

## API (High-Level)

See [API_DOCUMENTATION.md](API_DOCUMENTATION.md) for full schemas and examples.

- `POST /vault/initialize`
- `POST /vault/deposit`
- `POST /vault/withdraw`
- `GET /vault/balance/:user`
- `GET /vault/transactions/:user`
- `GET /vault/tvl`
- `WS /ws/vaults` (real-time updates)
