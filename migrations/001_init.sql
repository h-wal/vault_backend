CREATE TYPE transaction_type AS ENUM (
    'initialize',
    'deposit',
    'withdraw',
    'lock',
    'unlock',
    'transfer'
);


CREATE TABLE vaults (
    vault_pda           TEXT PRIMARY KEY,

    program_id          TEXT NOT NULL,
    network             TEXT NOT NULL, 

    owner_pubkey        TEXT NOT NULL,
    mint                TEXT NOT NULL,
    vault_token_account TEXT NOT NULL,

    total_balance       BIGINT NOT NULL,
    locked_balance      BIGINT NOT NULL,
    available_balance   BIGINT NOT NULL,

    total_deposited     BIGINT NOT NULL,
    total_withdrawn     BIGINT NOT NULL,

    created_at          TIMESTAMP NOT NULL,
    last_synced_at      TIMESTAMP NOT NULL
);

CREATE INDEX idx_vaults_owner ON vaults(owner_pubkey);
CREATE INDEX idx_vaults_network ON vaults(network);
CREATE INDEX idx_vaults_program ON vaults(program_id);


CREATE TABLE transactions (
    id              UUID PRIMARY KEY,

    vault_pda       TEXT NOT NULL,
    program_id      TEXT NOT NULL,
    network         TEXT NOT NULL,

    user_pubkey     TEXT,
    tx_signature    TEXT NOT NULL UNIQUE,

    tx_type         transaction_type NOT NULL,
    amount          BIGINT NOT NULL,

    slot            BIGINT NOT NULL,
    block_time      TIMESTAMP NOT NULL,

    created_at      TIMESTAMP DEFAULT now(),

    CONSTRAINT fk_transactions_vault
        FOREIGN KEY (vault_pda)
        REFERENCES vaults(vault_pda)
        ON DELETE CASCADE
);

CREATE INDEX idx_tx_vault ON transactions(vault_pda);
CREATE INDEX idx_tx_user ON transactions(user_pubkey);
CREATE INDEX idx_tx_network ON transactions(network);


CREATE TABLE balance_snapshots (
    vault_pda           TEXT NOT NULL,

    program_id          TEXT NOT NULL,
    network             TEXT NOT NULL,

    snapshot_time       TIMESTAMP NOT NULL,

    total_balance       BIGINT NOT NULL,
    locked_balance      BIGINT NOT NULL,
    available_balance   BIGINT NOT NULL,

    PRIMARY KEY (vault_pda, snapshot_time),

    CONSTRAINT fk_snapshots_vault
        FOREIGN KEY (vault_pda)
        REFERENCES vaults(vault_pda)
        ON DELETE CASCADE
);


CREATE TABLE processed_events (
    tx_signature   TEXT PRIMARY KEY,
    processed_at   TIMESTAMP NOT NULL DEFAULT now()
);


CREATE TABLE authorized_programs (
    program_id      TEXT PRIMARY KEY,
    admin_pubkey    TEXT NOT NULL,
    added_at        TIMESTAMP NOT NULL
);


CREATE TABLE program_calls (
    tx_signature    TEXT PRIMARY KEY,

    caller_program  TEXT NOT NULL,
    vault_pda       TEXT NOT NULL,

    instruction     TEXT NOT NULL,
    amount          BIGINT,

    slot            BIGINT NOT NULL,
    block_time      TIMESTAMP NOT NULL,

    CONSTRAINT fk_program_calls_vault
        FOREIGN KEY (vault_pda)
        REFERENCES vaults(vault_pda)
        ON DELETE CASCADE
);


CREATE TABLE reconciliation_logs (
    id                  UUID PRIMARY KEY,

    vault_pda           TEXT NOT NULL,
    program_id          TEXT NOT NULL,
    network             TEXT NOT NULL,

    onchain_balance     BIGINT NOT NULL,
    offchain_balance    BIGINT NOT NULL,
    discrepancy         BIGINT NOT NULL,

    detected_at         TIMESTAMP NOT NULL,
    resolved            BOOLEAN DEFAULT false,

    CONSTRAINT fk_reconciliation_vault
        FOREIGN KEY (vault_pda)
        REFERENCES vaults(vault_pda)
        ON DELETE CASCADE
);