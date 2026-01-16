// use solana_client::rpc_client::RpcClient;
// use solana_sdk::instruction::Instruction;
// use solana_sdk::{
//     pubkey::Pubkey,
//     signature::{Keypair, Signer},
//     system_program,
//     transaction::Transaction,
// };
// use solana_sdk::solana_program::system_instruction;
// use spl_associated_token_account::{
//     get_associated_token_address, instruction::create_associated_token_account,
// };
// use spl_token::solana_program::pubkey::Pubkey as ProgramPubkey;
// use spl_token::{solana_program::program_pack::Pack, state::Mint};
// use spl_token_interface::instruction as token_ix;
// use std::str::FromStr;

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let rpc_url = "http://127.0.0.1:8899".to_string();
//     let _program_id = Pubkey::from_str("9hhWr2GoSnXJmpaddFkgUFKfyG4fioZPf2GWtEGmQMWZ")?;

//     let user = Keypair::new();
//     let payer = Keypair::new();

//     let rpc = RpcClient::new(rpc_url.clone());

//     rpc.request_airdrop(&user.pubkey(), 10_000_000_000)?;
//     rpc.request_airdrop(&payer.pubkey(), 10_000_000_000)?;

//     loop {
//         let balance = rpc.get_balance(&payer.pubkey())?;
//         if balance > 0 {
//             break;
//         }
//         std::thread::sleep(std::time::Duration::from_millis(500));
//     }

//     let transfer_ix = system_instruction::transfer(&payer.pubkey(), &user.pubkey(), 1_000_000_000);

//     let blockhash = rpc.get_latest_blockhash()?;

//     let tx = Transaction::new_signed_with_payer(
//         &[transfer_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         blockhash,
//     );

//     rpc.send_and_confirm_transaction(&tx)?;

//     println!("System transfer successful");
//     println!("SPL Mint struct size constant exists");

//     let mint_size = Mint::get_packed_len();
//     println!("SPL Mint account size: {}", mint_size);

//     let mint = Keypair::new();
//     println!("mint pubkey: {}", mint.pubkey());

//     let rent = rpc.get_minimum_balance_for_rent_exemption(mint_size)?;

//     let token_program_id = Pubkey::new_from_array(spl_token::id().to_bytes());

//     let create_mint_account_ix = system_instruction::create_account(
//         &payer.pubkey(),
//         &mint.pubkey(),
//         rent,
//         mint_size as u64,
//         &token_program_id,
//     );

//     let blockhash = rpc.get_latest_blockhash()?;

//     let tx = Transaction::new_signed_with_payer(
//         &[create_mint_account_ix],
//         Some(&payer.pubkey()),
//         &[&payer, &mint],
//         blockhash,
//     );

//     rpc.send_and_confirm_transaction(&tx)?;

//     println!("mint creation successfull !");

//     let token_program_pubkey = ProgramPubkey::new_from_array(token_program_id.to_bytes());
//     let mint_pubkey = ProgramPubkey::new_from_array(mint.pubkey().to_bytes());
//     let authority_pubkey = ProgramPubkey::new_from_array(payer.pubkey().to_bytes());

//     let init_mint_ix = token_ix::initialize_mint(
//         &token_program_pubkey,
//         &mint_pubkey,
//         &authority_pubkey,
//         None,
//         6,
//     )?;

//     let init_mint_ix = Instruction {
//         program_id: token_program_id,
//         accounts: init_mint_ix
//             .accounts
//             .into_iter()
//             .map(|meta| solana_sdk::instruction::AccountMeta {
//                 pubkey: Pubkey::new_from_array(meta.pubkey.to_bytes()),
//                 is_signer: meta.is_signer,
//                 is_writable: meta.is_writable,
//             })
//             .collect(),
//         data: init_mint_ix.data,
//     };

//     let blockhash = rpc.get_latest_blockhash()?;

//     let tx = Transaction::new_signed_with_payer(
//         &[init_mint_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         blockhash,
//     );

//     rpc.send_and_confirm_transaction(&tx)?;

//     println!("mint initialized !");

//     //////////////////////////// setup done /////////////////////////////

//     let user_ata = get_associated_token_address(&user.pubkey(), &mint.pubkey());

//     println!("user ata: {}", user_ata);

//     let create_user_ata_ix = create_associated_token_account(
//         &payer.pubkey(),
//         &user.pubkey(),
//         &mint.pubkey(),
//         &token_program_id,
//     );

//     let blockhash = rpc.get_latest_blockhash()?;

//     let tx = Transaction::new_signed_with_payer(
//         &[create_user_ata_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         blockhash,
//     );

//     rpc.send_and_confirm_transaction(&tx)?;

//     println!("User ata creation successfull");

//     //////////////////////////// user ATA creation done /////////////////////////////

//     let mint_to_ix = token_ix::mint_to(
//         &token_program_pubkey,
//         &mint_pubkey,
//         &ProgramPubkey::new_from_array(user_ata.to_bytes()),
//         &authority_pubkey,
//         &[],
//         1_000_000_000,
//     )?;

//     let mint_to_ix = Instruction {
//         program_id: token_program_id,
//         accounts: mint_to_ix
//             .accounts
//             .into_iter()
//             .map(|meta| solana_sdk::instruction::AccountMeta {
//                 pubkey: Pubkey::new_from_array(meta.pubkey.to_bytes()),
//                 is_signer: meta.is_signer,
//                 is_writable: meta.is_writable,
//             })
//             .collect(),
//         data: mint_to_ix.data,
//     };

//     let blockhash = rpc.get_latest_blockhash()?;

//     let tx = Transaction::new_signed_with_payer(
//         &[mint_to_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         blockhash,
//     );

//     rpc.send_and_confirm_transaction(&tx)?;

//     println!("Tokens minted to ATA");

//     ////////////////////Tokens successfully minted to ATA //////////////////

//     let vault_authority = Keypair::new();
//     println!("vault_authority {}", vault_authority.pubkey());

//     let vault_ata = get_associated_token_address(&vault_authority.pubkey(), &mint.pubkey());

//     println!("vault_ata {}", vault_ata);

//     let create_vault_ata_ix = create_associated_token_account(
//         &payer.pubkey(),
//         &vault_authority.pubkey(),
//         &mint.pubkey(),
//         &token_program_id,
//     );

//     let blockhash = rpc.get_latest_blockhash()?;

//     let tx = Transaction::new_signed_with_payer(
//         &[create_vault_ata_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         blockhash,
//     );

//     rpc.send_and_confirm_transaction(&tx)?;
//     println!("Vault ATA created");

//     ////////////////////  Trasfering tokens from user to vault ata ////////////////////////

//     let transfer_ix = token_ix::transfer(
//         &token_program_pubkey,
//         &ProgramPubkey::new_from_array(user_ata.to_bytes()),
//         &ProgramPubkey::new_from_array(vault_ata.to_bytes()),
//         &ProgramPubkey::new_from_array(user.pubkey().to_bytes()),
//         &[],
//         500_000_000,
//     )?;

//     let transfer_ix = Instruction {
//         program_id: token_program_id,
//         accounts: transfer_ix
//             .accounts
//             .into_iter()
//             .map(|meta| solana_sdk::instruction::AccountMeta {
//                 pubkey: Pubkey::new_from_array(meta.pubkey.to_bytes()),
//                 is_signer: meta.is_signer,
//                 is_writable: meta.is_writable,
//             })
//             .collect(),
//         data: transfer_ix.data,
//     };

//     let blockhash = rpc.get_latest_blockhash()?;

//     let tx = Transaction::new_signed_with_payer(
//         &[transfer_ix],
//         Some(&payer.pubkey()),
//         &[&payer, &user],
//         blockhash,
//     );

//     rpc.send_and_confirm_transaction(&tx)?;

//     print!("deposit successfull!");

//     // let _vault_manager = VaultManager::new(rpc_url, program_id, payer);

//     Ok(())
// }

fn main(){
    println!("Test Scirpt !");
}