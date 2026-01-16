use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

/// Event types mirrored from the on-chain Anchor program.
///
/// These structs are only used for Borsh deserialization of event payloads
/// in the indexer. They intentionally keep a minimal surface area: fields
/// map 1:1 to what `event_decoder` needs.

#[derive(BorshDeserialize)]
pub struct VaultAuthorityInitialized {
    pub admin: Pubkey,
}

#[derive(BorshDeserialize)]
pub struct ProgramAuthorized {
    pub program_id: Pubkey,
}

#[derive(BorshDeserialize)]
pub struct VaultInitialized {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub timestamp: i64,
}

#[derive(BorshDeserialize)]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[derive(BorshDeserialize)]
pub struct CollateralWithdrawn {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}

#[derive(BorshDeserialize)]
pub struct CollateralLocked {
    pub vault: Pubkey,
    pub amount: u64,
}

#[derive(BorshDeserialize)]
pub struct CollateralUnlocked {
    pub vault: Pubkey,
    pub amount: u64,
}

#[derive(BorshDeserialize)]
pub struct CollateralTransferred {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}
