use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;

const TOKEN_2022_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub struct TransactionBuilder {
    program_id: Pubkey,
}

impl TransactionBuilder {
    pub fn new(program_id: Pubkey) -> Self {
        Self { program_id }
    }

    pub fn derive_vault_pda(&self, user: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program_id)
    }

    pub fn build_deposit_ix(
        &self,
        user: &Pubkey,
        mint: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<Instruction> {
        let (vault_pda, _) = self.derive_vault_pda(user); // calculating the pda of the vault using the fn defind above

        let user_token_account =
            get_associated_token_address_with_program_id(user, mint, &TOKEN_2022_PROGRAM_ID);

        let vault_token_account =
            get_associated_token_address_with_program_id(&vault_pda, mint, &TOKEN_2022_PROGRAM_ID);

        let discriminator: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182]; //from the idl
        let mut data = discriminator.to_vec(); // convert to a vector
        data.extend_from_slice(&amount.to_le_bytes()); // adding the amount at the end to form instruction

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
}
