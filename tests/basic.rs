extern crate solana_program;
extern crate solana_sdk;
extern crate spl_token_client;

use solana_client::rpc_client::RpcClient;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::{Keypair, Signer};
use spl_token_client::client::{ProgramClient, ProgramRpcClient, ProgramRpcClientSendTransaction};
use spl_token_client::token::Token;
use std::sync::Arc;

#[tokio::test]
async fn basic() -> Result<(), Box<dyn std::error::Error>> {
    let existing_token_mint = Keypair::new();
    let client = RpcClient::new("http://localhost:8899".to_string());
    client.request_airdrop(&existing_token_mint.pubkey(), LAMPORTS_PER_SOL * 10)?;

    let mint_account_1 = Keypair::new();
    let program_client = Arc::new(ProgramRpcClient::new(
        &client,
        ProgramRpcClientSendTransaction,
    ));

    Token::create_mint(
        program_client,
        Keypair::from_bytes(&existing_token_mint.to_bytes())?,
        &mint_account_1,
        &existing_token_mint.pubkey(),
        None,
        9,
    )
    .await?;

    Ok(())
}
