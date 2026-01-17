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
    program_id: Pubkey, // this program id it public key of the user.
}

impl TransactionBuilder {

    pub fn new(program_id: Pubkey) -> Self { // this is the factory function for the transaction builder
        Self { program_id }
    }


    pub fn derive_vault_pda(&self, user: &Pubkey) -> (Pubkey, u8) { // this is the function to derive the vault pda from the user pubkey
        Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program_id)
    }

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

        let discriminator: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182]; // this is the discriminator for the deposit instruction extracted from the idl 
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

        let discriminator: [u8; 8] = [48, 191, 163, 44, 71, 129, 63, 164]; // this is the discriminator for initialize vault instruction from our idl

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

     // derive the vault_authority pda used for lock / unlock collateral trasaction
    pub fn derive_vault_authority_pda(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vault_authority"], &self.program_id)
    }

    pub fn build_lock_collateral_ix( // transaction to lock collateral from avaialible to locked collateral vault 
        &self,
        caller_program: &Pubkey,
        user: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<Instruction> {
        let (vault_pda, _) = self.derive_vault_pda(user);
        let (vault_authority_pda, _) = self.derive_vault_authority_pda();

        // Discriminator from IDL: lock_collateral = [161, 216, 135, 122, 12, 104, 211, 101]
        let discriminator: [u8; 8] = [161, 216, 135, 122, 12, 104, 211, 101];
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new_readonly(*caller_program, false), // caller program (checked for authorization)
            AccountMeta::new(vault_pda, false),                // vault PDA (mutable)
            AccountMeta::new_readonly(vault_authority_pda, false), // vault authority PDA (read-only for validation)
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,   
            data,
        })
    }


    pub fn build_unlock_collateral_ix( // used to unlock collateral from locked vaule to availaible value
        &self,
        caller_program: &Pubkey,
        user: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<Instruction> {
        let (vault_pda, _) = self.derive_vault_pda(user);
        let (vault_authority_pda, _) = self.derive_vault_authority_pda();

        // Discriminator from IDL: unlock_collateral = [167, 213, 221, 147, 129, 209, 132, 190]
        let discriminator: [u8; 8] = [167, 213, 221, 147, 129, 209, 132, 190];
        let mut data = discriminator.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new_readonly(*caller_program, false), // caller program (checked for authorization)
            AccountMeta::new(vault_pda, false),                // vault PDA (mutable)
            AccountMeta::new_readonly(vault_authority_pda, false), // vault authority PDA (read-only for validation)
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

}
