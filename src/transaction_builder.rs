use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_associated_token_account::get_associated_token_address_with_program_id;

const TOKEN_2022_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

// Builds Solana transactions for vault operations
pub struct TransactionBuilder {
    program_id: Pubkey,
}

impl TransactionBuilder {
    pub fn new(program_id: Pubkey) -> Self {
        Self { program_id }
    }

    // Derive the vault's program-derived address (PDA) from user pubkey
    pub fn derive_vault_pda(&self, user: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program_id)
    }

    // Build an instruction to deposit tokens into a vault
    pub fn build_deposit_ix(
        &self,
        user: &Pubkey,
        mint: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<Instruction> {
        let (vault_pda, _) = self.derive_vault_pda(user);

        let user_token_account =
            get_associated_token_address_with_program_id(user, mint, &TOKEN_2022_PROGRAM_ID);

        let vault_token_account =
            get_associated_token_address_with_program_id(&vault_pda, mint, &TOKEN_2022_PROGRAM_ID);

        // Instruction discriminator from the program IDL
        let discriminator: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    pub fn build_initialize_vault_ix(
        &self,
        user: &Pubkey,
        mint: &Pubkey,
    ) -> anyhow::Result<Instruction> {
        let (vault_pda, vault_bump) = self.derive_vault_pda(user);

        let vault_token_account =
            get_associated_token_address_with_program_id(&vault_pda, mint, &TOKEN_2022_PROGRAM_ID);

        let discriminator: [u8; 8] = [48, 191, 163, 44, 71, 129, 63, 164];

        let mut data = discriminator.to_vec();
        data.push(vault_bump);

        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    pub fn build_withdraw_ix(
        &self,
        user: &Pubkey,
        mint: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<Instruction> {
        let (vault_pda, _) = self.derive_vault_pda(user);

        let vault_token_account =
            get_associated_token_address_with_program_id(&vault_pda, mint, &TOKEN_2022_PROGRAM_ID);

        let user_token_account =
            get_associated_token_address_with_program_id(user, mint, &TOKEN_2022_PROGRAM_ID);

        let discriminator: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];

        let mut data = discriminator.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(*user, true),      // user signer
            AccountMeta::new(vault_pda, false), // vault PDA
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

}
