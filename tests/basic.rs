extern crate solana_program;
extern crate solana_sdk;

use solana_client::rpc_client::RpcClient;
use solana_program::message::Message;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::program_pack::Pack;
use solana_program::system_instruction;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_token::instruction::initialize_mint;
use spl_token::state::Mint;
use std::sync::Arc;

// TODO create caller account with sufficient sol

// TODO create 2 input tokens
// TODO create empty input accounts for smart contract
// TODO create input token accounts with sufficient balance for the caller
// TODO create 2 output tokens
// TODO create output token accounts with entire supply for smart contract
// TODO create empty output token accounts for caller
// TODO initialize tokenitis smart contract
// TODO execute tokenitis smart contract
// TODO validate outputs

#[test]
fn basic() -> Result<(), Box<dyn std::error::Error>> {
    let client: RpcClient = RpcClient::new("http://localhost:8899".to_string());

    // Create account that will test the functionality of the smart contract
    let user = Keypair::new();
    let sig = client.request_airdrop(&user.pubkey(), LAMPORTS_PER_SOL * 10)?;
    loop {
        if client.confirm_transaction(&sig)? {
            break;
        }
    }
    let user_balance = client.get_balance(&user.pubkey())?;
    println!(
        "user account created with pubkey - {}, with balance - {}",
        user.pubkey(),
        user_balance
    );

    // Create input token mints
    let minimum_balance_for_rent_exemption =
        client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;
    let input_token_1_mint = Keypair::new();
    let instructions = vec![
        system_instruction::create_account(
            &user.pubkey(),
            &input_token_1_mint.pubkey(),
            minimum_balance_for_rent_exemption,
            Mint::LEN as u64,
            &spl_token::ID,
        ),
        initialize_mint(
            &spl_token::ID,
            &input_token_1_mint.pubkey(),
            &user.pubkey(),
            None,
            9,
        )?,
    ];
    let msg = Message::new(instructions.as_slice(), Some(&user.pubkey()));
    let blockhash = client.get_latest_blockhash()?;
    let signers: Vec<&dyn Signer> = vec![&user, &input_token_1_mint];
    let tx = Transaction::new(&signers, msg, blockhash);
    let sig = client.send_and_confirm_transaction_with_spinner(&tx)?;
    loop {
        if client.confirm_transaction(&sig)? {
            break;
        }
    }

    println!("created token mint - {}", sig);

    Ok(())
}
