# Vault Backend API Documentation

## Overview
REST API for managing Solana collateral vaults. Provides endpoints for vault initialization, deposits, withdrawals, balance queries, and transaction history.

## Base URL
```
http://localhost:8080
```

## Authentication
All requests require wallet signature verification (implemented at transport layer).

---

## Endpoints

### 1. Initialize Vault
**POST** `/vault/initialize`

Initialize a new vault for a user.

**Request Body:**
```json
{
  "user_pubkey": "string (Solana public key)",
  "mint": "string (SPL token mint address)"
}
```

**Response (200 OK):**
```json
{
  "status": "success",
  "vault_pda": "string (derived PDA)",
  "transaction_signature": "string"
}
```

**Errors:**
- `400 Bad Request`: Invalid public key format
- `409 Conflict`: Vault already exists for user
- `500 Internal Server Error`: Transaction failed

---

### 2. Deposit to Vault
**POST** `/vault/deposit`

Deposit collateral into an existing vault.

**Request Body:**
```json
{
  "user_pubkey": "string",
  "mint": "string",
  "amount": "number (in smallest token units)"
}
```

**Response (200 OK):**
```json
{
  "status": "success",
  "transaction_signature": "string",
  "new_balance": "number",
  "timestamp": "ISO 8601 datetime"
}
```

**Errors:**
- `400 Bad Request`: Invalid amount (must be > 0)
- `404 Not Found`: Vault not found
- `422 Unprocessable Entity`: Insufficient token balance
- `500 Internal Server Error`: Transaction failed

---

### 3. Withdraw from Vault
**POST** `/vault/withdraw`

Withdraw collateral from vault.

**Request Body:**
```json
{
  "user_pubkey": "string",
  "mint": "string",
  "amount": "number"
}
```

**Response (200 OK):**
```json
{
  "status": "success",
  "transaction_signature": "string",
  "remaining_balance": "number",
  "timestamp": "ISO 8601 datetime"
}
```

**Errors:**
- `400 Bad Request`: Invalid amount or withdrawal locked
- `404 Not Found`: Vault not found
- `422 Unprocessable Entity`: Insufficient available balance
- `500 Internal Server Error`: Transaction failed

---

### 4. Get Vault Balance
**GET** `/vault/balance/:user`

Retrieve current vault balance and status.

**Path Parameters:**
- `user` (string): User's Solana public key

**Response (200 OK):**
```json
{
  "user_pubkey": "string",
  "vault_pda": "string",
  "total_balance": "number",
  "locked_balance": "number",
  "available_balance": "number",
  "last_synced_at": "ISO 8601 datetime"
}
```

**Errors:**
- `400 Bad Request`: Invalid public key
- `404 Not Found`: Vault not found
- `500 Internal Server Error`: Query failed

---

### 5. Get Transaction History
**GET** `/vault/transactions/:user`

Retrieve transaction history for a vault.

**Path Parameters:**
- `user` (string): User's Solana public key

**Query Parameters:**
- `limit` (number, optional): Number of transactions to return (default: 50, max: 1000)
- `offset` (number, optional): Pagination offset (default: 0)
- `type` (string, optional): Filter by transaction type (initialize, deposit, withdraw, lock, unlock, transfer)

**Response (200 OK):**
```json
{
  "total_count": "number",
  "transactions": [
    {
      "id": "string (UUID)",
      "type": "string (deposit|withdraw|lock|unlock|transfer)",
      "amount": "number",
      "signature": "string (transaction signature)",
      "status": "string (confirmed|failed|pending)",
      "timestamp": "ISO 8601 datetime"
    }
  ]
}
```

**Errors:**
- `400 Bad Request`: Invalid parameters
- `404 Not Found`: Vault not found
- `500 Internal Server Error`: Query failed

---

### 6. Get Total Value Locked (TVL)
**GET** `/vault/tvl`

Get total value locked across all vaults.

**Query Parameters:**
- `mint` (string, optional): Filter by specific mint

**Response (200 OK):**
```json
{
  "total_value_locked": "number",
  "vault_count": "number",
  "top_vaults": [
    {
      "user": "string",
      "balance": "number",
      "percentage": "number"
    }
  ],
  "timestamp": "ISO 8601 datetime"
}
```

**Errors:**
- `500 Internal Server Error`: Calculation failed

---

## WebSocket Streams

### Real-time Vault Updates
**WS** `/ws/vaults`

Stream real-time updates for vault events.

**Message Types:**

**TVL Update:**
```json
{
  "type": "tvl_update",
  "tvl": "number",
  "timestamp": "ISO 8601 datetime"
}
```

**Balance Update:**
```json
{
  "type": "balance_update",
  "user": "string",
  "new_balance": "number",
  "available_balance": "number",
  "timestamp": "ISO 8601 datetime"
}
```

**Transaction Event:**
```json
{
  "type": "transaction",
  "user": "string",
  "tx_type": "string",
  "amount": "number",
  "signature": "string",
  "timestamp": "ISO 8601 datetime"
}
```

---

## Common Errors

### 400 Bad Request
Invalid input parameters or malformed request.
```json
{
  "error": "string",
  "details": "string"
}
```

### 401 Unauthorized
Missing or invalid authentication.
```json
{
  "error": "Unauthorized",
  "message": "Invalid signature"
}
```

### 404 Not Found
Resource doesn't exist.
```json
{
  "error": "Not Found",
  "message": "Vault not found for user"
}
```

### 429 Too Many Requests
Rate limit exceeded.
```json
{
  "error": "Too Many Requests",
  "retry_after": "number (seconds)"
}
```

### 500 Internal Server Error
Server-side error.
```json
{
  "error": "Internal Server Error",
  "request_id": "string (for debugging)"
}
```

---

## Rate Limiting

- **Limit**: 1000 requests per minute per user
- **Headers**:
  - `X-RateLimit-Limit`: Total requests allowed
  - `X-RateLimit-Remaining`: Requests remaining
  - `X-RateLimit-Reset`: Unix timestamp when limit resets

---

## Data Types

### Transaction Status
- `pending`: Transaction submitted, awaiting confirmation
- `confirmed`: Transaction confirmed on-chain
- `failed`: Transaction failed

### Transaction Type
- `initialize`: Vault initialization
- `deposit`: Collateral deposit
- `withdraw`: Collateral withdrawal
- `lock`: Collateral locked for position
- `unlock`: Collateral unlocked from position
- `transfer`: Internal transfer

---

## Examples

### Initialize Vault
```bash
curl -X POST http://localhost:8080/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "7hWJ...kxyz",
    "mint": "EPjFWdd...abc"
  }'
```

### Deposit Funds
```bash
curl -X POST http://localhost:8080/vault/deposit \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "7hWJ...kxyz",
    "mint": "EPjFWdd...abc",
    "amount": 1000000000
  }'
```

### Get Balance
```bash
curl http://localhost:8080/vault/balance/7hWJ...kxyz
```

### Watch Real-time Updates
```bash
wscat -c ws://localhost:8080/ws/vaults
```

---

## Deployment

The API is deployed with:
- **Framework**: Axum (async Rust web framework)
- **Database**: PostgreSQL with connection pooling
- **RPC**: Solana RPC cluster
- **Port**: 8080 (configurable via `API_PORT` env var)

---

## Environment Variables

```env
# API Configuration
API_PORT=8080
API_WORKERS=4

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/vault_db

# Solana RPC
SOLANA_RPC_URL=http://localhost:8899
SOLANA_PROGRAM_ID=9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ

# Logging
RUST_LOG=info,vault_backend=debug

# Security
ENABLE_AUTH=true
RATE_LIMIT_ENABLED=true
```

---

## Support

For issues or questions:
1. Check logs: `docker logs vault-backend`
2. Review error messages with `request_id`
3. Contact the development team
