extern crate solana_program;
extern crate solana_sdk;

use solana_client::rpc_client::RpcClient;
use solana_program::instruction::Instruction;
use solana_program::message::Message;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use spl_token::instruction::{initialize_account, initialize_mint, mint_to_checked};
use spl_token::state::{Account, Mint};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

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
        sleep(Duration::from_millis(500))
    }
    let user_balance = client.get_balance(&user.pubkey())?;
    println!(
        "user account created with pubkey - {}, with balance - {}",
        user.pubkey(),
        user_balance
    );

    // Create input token mints
    let input_token_decimals: u8 = 9;
    let input_token_1_mint = Keypair::new();
    let sig = create_token_mint(&client, &user, &input_token_1_mint, input_token_decimals)?;
    println!(
        "created input token 1 mint, with pubkey - {}",
        input_token_1_mint.pubkey()
    );
    let input_token_2_mint = Keypair::new();
    let sig = create_token_mint(&client, &user, &input_token_2_mint, input_token_decimals)?;
    println!(
        "created input token 2 mint, with pubkey - {}",
        input_token_2_mint.pubkey()
    );

    // Creating input token user accounts
    let input_token_1_user_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &input_token_1_mint,
        &input_token_1_user_account,
    )?;
    let sig = mint_to_token_account(
        &client,
        &user,
        &input_token_1_mint,
        &input_token_1_user_account,
        100,
        input_token_decimals,
    );
    let balance = client.get_token_account_balance(&input_token_1_user_account.pubkey())?;
    println!(
        "created input token 1 user account, with pubkey - {}, with balance - {}",
        input_token_1_user_account.pubkey(),
        balance.amount
    );

    let input_token_2_user_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &input_token_2_mint,
        &input_token_2_user_account,
    )?;
    let sig = mint_to_token_account(
        &client,
        &user,
        &input_token_2_mint,
        &input_token_2_user_account,
        100,
        input_token_decimals,
    );
    let balance = client.get_token_account_balance(&input_token_2_user_account.pubkey())?;
    println!(
        "created input token 2 user account, with pubkey - {}, with balance - {}",
        input_token_2_user_account.pubkey(),
        balance.amount
    );

    // Creating input token smart contract accounts
    let input_token_1_sc_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &input_token_1_mint,
        &input_token_1_sc_account,
    )?;
    println!(
        "created input token 1 sc account, with pubkey - {}",
        input_token_1_sc_account.pubkey(),
    );

    let input_token_2_sc_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &input_token_2_mint,
        &input_token_2_sc_account,
    )?;
    println!(
        "created input token 2 sc account, with pubkey - {}",
        input_token_2_sc_account.pubkey(),
    );

    // Create output token mints
    let output_token_decimal: u8 = 9;
    let output_token_1_mint = Keypair::new();
    let sig = create_token_mint(&client, &user, &output_token_1_mint, output_token_decimal)?;
    println!(
        "created output token 1 mint, with pubkey - {}",
        output_token_1_mint.pubkey()
    );
    let output_token_2_mint = Keypair::new();
    let sig = create_token_mint(&client, &user, &output_token_2_mint, output_token_decimal)?;
    println!(
        "created output token 2 mint, with pubkey - {}",
        output_token_2_mint.pubkey()
    );

    // Creating output token sc accounts
    let output_token_1_sc_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &output_token_1_mint,
        &output_token_1_sc_account,
    )?;
    let sig = mint_to_token_account(
        &client,
        &user,
        &output_token_1_mint,
        &output_token_1_sc_account,
        100,
        output_token_decimal,
    );
    let balance = client.get_token_account_balance(&output_token_1_sc_account.pubkey())?;
    println!(
        "created output token 1 sc account, with pubkey - {}, with balance - {}",
        output_token_1_sc_account.pubkey(),
        balance.amount
    );

    let output_token_2_sc_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &output_token_2_mint,
        &output_token_2_sc_account,
    )?;
    let sig = mint_to_token_account(
        &client,
        &user,
        &output_token_2_mint,
        &output_token_2_sc_account,
        100,
        output_token_decimal,
    );
    let balance = client.get_token_account_balance(&output_token_2_sc_account.pubkey())?;
    println!(
        "created output token 2 sc account, with pubkey - {}, with balance - {}",
        output_token_2_sc_account.pubkey(),
        balance.amount
    );

    // Creating output token user accounts
    let output_token_1_user_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &output_token_1_mint,
        &output_token_1_user_account,
    )?;
    println!(
        "created output token 1 user account, with pubkey - {}",
        output_token_1_user_account.pubkey(),
    );

    let output_token_2_user_account = Keypair::new();
    let sig = create_token_account(
        &client,
        &user,
        &output_token_2_mint,
        &output_token_2_user_account,
    )?;
    println!(
        "created output token 2 user account, with pubkey - {}",
        output_token_2_user_account.pubkey(),
    );

    Ok(())
}

fn initialize_tokenitis() -> Result<Signature, Box<dyn std::error::Error>> {
    let instructions = vec![mint_to_checked(
        &spl_token::ID,
        &mint_account.pubkey(),
        &token_account.pubkey(),
        &mint_authority.pubkey(),
        &[&mint_authority.pubkey()],
        amount,
        decimals,
    )?];
    let signers: Vec<&dyn Signer> = vec![mint_authority];
    let sig = create_and_send_tx(
        &client,
        instructions,
        signers,
        Some(&mint_authority.pubkey()),
    )?;

    Ok(sig)
}

fn mint_to_token_account(
    client: &RpcClient,
    mint_authority: &Keypair,
    mint_account: &Keypair,
    token_account: &Keypair,
    amount: u64,
    decimals: u8,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let instructions = vec![mint_to_checked(
        &spl_token::ID,
        &mint_account.pubkey(),
        &token_account.pubkey(),
        &mint_authority.pubkey(),
        &[&mint_authority.pubkey()],
        amount,
        decimals,
    )?];
    let signers: Vec<&dyn Signer> = vec![mint_authority];
    let sig = create_and_send_tx(
        &client,
        instructions,
        signers,
        Some(&mint_authority.pubkey()),
    )?;

    Ok(sig)
}

fn create_token_account(
    client: &RpcClient,
    owner: &Keypair,
    mint_account: &Keypair,
    token_account: &Keypair,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let minimum_balance_for_rent_exemption =
        client.get_minimum_balance_for_rent_exemption(Account::LEN)?;

    let instructions = vec![
        system_instruction::create_account(
            &owner.pubkey(),
            &token_account.pubkey(),
            minimum_balance_for_rent_exemption,
            Account::LEN as u64,
            &spl_token::ID,
        ),
        initialize_account(
            &spl_token::ID,
            &token_account.pubkey(),
            &mint_account.pubkey(),
            &owner.pubkey(),
        )?,
    ];
    let signers: Vec<&dyn Signer> = vec![owner, token_account];
    let sig = create_and_send_tx(&client, instructions, signers, Some(&owner.pubkey()))?;

    Ok(sig)
}

fn create_token_mint(
    client: &RpcClient,
    mint_authority: &Keypair,
    mint_account: &Keypair,
    decimals: u8,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let minimum_balance_for_rent_exemption =
        client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

    let instructions = vec![
        system_instruction::create_account(
            &mint_authority.pubkey(),
            &mint_account.pubkey(),
            minimum_balance_for_rent_exemption,
            Mint::LEN as u64,
            &spl_token::ID,
        ),
        initialize_mint(
            &spl_token::ID,
            &mint_account.pubkey(),
            &mint_authority.pubkey(),
            None,
            decimals,
        )?,
    ];
    let signers: Vec<&dyn Signer> = vec![mint_authority, mint_account];
    let sig = create_and_send_tx(
        &client,
        instructions,
        signers,
        Some(&mint_authority.pubkey()),
    )?;

    Ok(sig)
}

fn create_and_send_tx(
    client: &RpcClient,
    instructions: Vec<Instruction>,
    signers: Vec<&dyn Signer>,
    payer: Option<&Pubkey>,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let msg = Message::new(instructions.as_slice(), payer);
    let tx = Transaction::new(&signers, msg, client.get_latest_blockhash()?);
    let sig = client.send_and_confirm_transaction(&tx)?;
    loop {
        if client.confirm_transaction(&sig)? {
            break;
        }
        sleep(Duration::from_millis(500))
    }

    Ok(sig)
}
