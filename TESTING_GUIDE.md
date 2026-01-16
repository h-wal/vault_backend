# Testing Guide for Vault Backend

This guide covers all the ways to test your backend, from unit tests to API integration tests.

## Table of Contents

1. [Unit Tests](#unit-tests)
2. [API Integration Testing](#api-integration-testing)
3. [Manual API Testing](#manual-api-testing)
4. [Environment Setup](#environment-setup)
5. [Test Script](#test-script)

---

## Unit Tests

Your project already has comprehensive unit tests. Run them with:

### Run All Unit Tests
```bash
cargo test
```

### Run Specific Test Suites
```bash
# Vault manager tests
cargo test vault_manager_tests

# Error handling tests
cargo test error_handling

# Access control tests
cargo test access_control

# Logging tests
cargo test logging
```

### Run Tests with Output
```bash
# Show println! output
cargo test -- --nocapture

# Run tests in sequence (useful for tests that might interfere with each other)
cargo test -- --test-threads=1

# Run a specific test
cargo test test_vault_manager_creation
```

### Expected Test Results
You should see 32+ tests passing:
- ✅ 13 vault manager tests
- ✅ 7 error handling tests
- ✅ 6 access control tests
- ✅ 3+ logging tests
- ✅ Other module tests

---

## API Integration Testing

To test the actual API endpoints, you'll need to run the server and make HTTP requests.

### 1. Start the Server

First, set up your environment variables:

```bash
# Create a .env file or export these:
export RPC_URL=http://127.0.0.1:8899  # Local Solana validator
export PROGRAM_ID=9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ
export DATABASE_URL=postgresql://user:password@localhost:5432/vault_db
export RUST_LOG=info,vault_backend=debug
```

Then start the server:
```bash
cargo run --bin server
```

The server will start on `http://0.0.0.0:8080`

### 2. Test with curl

#### Initialize a Vault
```bash
curl -X POST http://localhost:8080/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "YOUR_USER_PUBKEY",
    "mint": "YOUR_MINT_PUBKEY"
  }'
```

#### Deposit Tokens
```bash
curl -X POST http://localhost:8080/vault/deposit \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "YOUR_USER_PUBKEY",
    "mint": "YOUR_MINT_PUBKEY",
    "amount": 1000000000
  }'
```

#### Get Balance
```bash
curl http://localhost:8080/vault/balance/YOUR_USER_PUBKEY
```

#### Get Transactions
```bash
curl http://localhost:8080/vault/transactions/YOUR_USER_PUBKEY
```

#### Get TVL (Total Value Locked)
```bash
curl http://localhost:8080/vault/tvl
```

#### Withdraw Tokens
```bash
curl -X POST http://localhost:8080/vault/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "YOUR_USER_PUBKEY",
    "mint": "YOUR_MINT_PUBKEY",
    "amount": 500000000
  }'
```

---

## Manual API Testing

### Using a REST Client (Postman, Insomnia, etc.)

1. **Create a new request collection** for vault operations

2. **Base URL**: `http://localhost:8080`

3. **Test Endpoints**:

   - `POST /vault/initialize`
   - `POST /vault/deposit`
   - `POST /vault/withdraw`
   - `GET /vault/balance/:user`
   - `GET /vault/transactions/:user`
   - `GET /vault/tvl`
   - `WS /ws/vaults` (WebSocket)

4. **Example Request Body** (for POST endpoints):
   ```json
   {
     "user_pubkey": "11111111111111111111111111111111",
     "mint": "22222222222222222222222222222222",
     "amount": 1000000000
   }
   ```

### Using WebSocket Client

Connect to `ws://localhost:8080/ws/vaults` to receive real-time TVL updates every 5 seconds.

You can use:
- Browser DevTools WebSocket client
- `websocat`: `websocat ws://localhost:8080/ws/vaults`
- Online WebSocket testers (websocket.org/echo.html)

---

## Environment Setup

### Prerequisites

1. **PostgreSQL Database**
   ```bash
   # Install PostgreSQL (if not already installed)
   # macOS: brew install postgresql
   # Ubuntu: sudo apt-get install postgresql
   
   # Create database
   createdb vault_db
   
   # Run migrations
   # You'll need to apply migrations/001_init.sql to your database
   psql vault_db < migrations/001_init.sql
   ```

2. **Solana Local Validator** (for testing)
   ```bash
   # Install Solana CLI tools
   sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
   
   # Start local validator
   solana-test-validator
   ```

3. **Environment Variables**
   ```bash
   # Create .env file in project root
   cat > .env << EOF
   RPC_URL=http://127.0.0.1:8899
   PROGRAM_ID=9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ
   DATABASE_URL=postgresql://user:password@localhost:5432/vault_db
   RUST_LOG=info,vault_backend=debug
   EOF
   ```

---

## Automated Integration Tests (Optional)

If you want to add automated integration tests, you can create a test file like `tests/integration_tests.rs`:

```rust
// This would go in tests/integration_tests.rs
// Note: You may need to add tower-http with features = ["util"] to Cargo.toml

use axum::http::StatusCode;
use vault_backend::api::{AppState, router};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tower::ServiceExt; // for `.oneshot()`
use std::str::FromStr;

#[tokio::test]
async fn test_health_check() {
    // This is a template - you'd need to set up test state properly
    // let state = create_test_state();
    // let app = router(state);
    // 
    // let response = app
    //     .oneshot(
    //         Request::builder()
    //             .uri("/vault/tvl")
    //             .body(Body::empty())
    //             .unwrap(),
    //     )
    //     .await
    //     .unwrap();
    // 
    // assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Test Script

You have a `test_script.rs` file that's currently commented out. It contains integration test code for Solana operations. To use it:

1. **Uncomment the code** in `src/bin/test_script.rs`
2. **Run it**:
   ```bash
   # Make sure your local Solana validator is running
   cargo run --bin test_script
   ```

This script tests:
- System transfers
- SPL token mint creation
- Associated token account creation
- Token minting
- Token transfers

---

## Debugging Tests

### Enable Detailed Logging
```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Run Tests with Backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Check if Server is Running
```bash
# Test server health
curl http://localhost:8080/vault/tvl
```

---

## Common Issues

### Tests Fail with Database Connection Error
- Make sure PostgreSQL is running: `pg_isready`
- Check `DATABASE_URL` is correct
- Ensure database exists and migrations are applied

### Tests Fail with RPC Connection Error
- Make sure Solana validator is running: `solana-test-validator`
- Check `RPC_URL` points to correct endpoint
- Verify validator is accessible at the URL

### API Requests Return 500 Errors
- Check server logs for error details
- Verify all environment variables are set
- Ensure database tables exist (run migrations)

---

## Next Steps

1. ✅ Run unit tests: `cargo test`
2. ✅ Start the server: `cargo run --bin server`
3. ✅ Test API endpoints manually with curl or Postman
4. ✅ Optionally add automated integration tests

For more details, see:
- [API_DOCUMENTATION.md](API_DOCUMENTATION.md) - Full API reference
- [README.md](README.md) - General project overview
