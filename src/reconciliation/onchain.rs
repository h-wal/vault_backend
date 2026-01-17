use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

/// Fetch SPL token balance for a token account
pub fn fetch_token_balance(
    rpc: &RpcClient,
    token_account: &Pubkey,
) -> anyhow::Result<u64> {
    let account = rpc.get_account(token_account)?;
    let token = TokenAccount::unpack(&account.data)?;
    Ok(token.amount)
}
