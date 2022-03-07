extern crate solana_program;
extern crate solana_sdk;

use borsh::BorshSerialize;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::message::Message;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use spl_token::instruction::{initialize_account, initialize_mint, mint_to_checked};
use spl_token::state::{Account, Mint};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokenitis::execute::{Direction, ExecuteArgs};
use tokenitis::initialize::InitializeArgs;
use tokenitis::instruction::TokenitisInstructions;
use tokenitis::state::{Tokenitis, SEED};

#[test]
fn basic() -> Result<(), Box<dyn std::error::Error>> {
    let client: RpcClient = RpcClient::new("http://localhost:8899".to_string());

    // Create account that will test the functionality of the smart contract
    let user = Keypair::new();
    let sig = client.request_airdrop(&user.pubkey(), LAMPORTS_PER_SOL * 10)?;
    confirm_transactions(&client, vec![sig])?;
    let user_balance = client.get_balance(&user.pubkey())?;
    println!(
        "user account created with pubkey - {}, with balance - {}",
        user.pubkey(),
        user_balance
    );

    // Create mints
    let input_token_decimals: u8 = 9;
    let input_token_1_mint = Keypair::new();
    let sig1 = create_token_mint(&client, &user, &input_token_1_mint, input_token_decimals)?;
    println!(
        "created input token 1 mint, with pubkey - {}",
        input_token_1_mint.pubkey()
    );
    let input_token_2_mint = Keypair::new();
    let sig2 = create_token_mint(&client, &user, &input_token_2_mint, input_token_decimals)?;
    println!(
        "created input token 2 mint, with pubkey - {}",
        input_token_2_mint.pubkey()
    );
    let output_token_decimal: u8 = 9;
    let output_token_1_mint = Keypair::new();
    let sig3 = create_token_mint(&client, &user, &output_token_1_mint, output_token_decimal)?;
    println!(
        "created output token 1 mint, with pubkey - {}",
        output_token_1_mint.pubkey()
    );
    let output_token_2_mint = Keypair::new();
    let sig4 = create_token_mint(&client, &user, &output_token_2_mint, output_token_decimal)?;
    println!(
        "created output token 2 mint, with pubkey - {}",
        output_token_2_mint.pubkey()
    );

    confirm_transactions(&client, vec![sig1, sig2, sig3, sig4])?;

    // Create token accounts
    let input_token_1_user_account = Keypair::new();
    let sig1 = create_token_account(
        &client,
        &user,
        &input_token_1_mint,
        &input_token_1_user_account,
    )?;
    let input_token_2_user_account = Keypair::new();
    let sig2 = create_token_account(
        &client,
        &user,
        &input_token_2_mint,
        &input_token_2_user_account,
    )?;
    let input_token_1_sc_account = Keypair::new();
    let sig3 = create_token_account(
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
    let sig4 = create_token_account(
        &client,
        &user,
        &input_token_2_mint,
        &input_token_2_sc_account,
    )?;
    println!(
        "created input token 2 sc account, with pubkey - {}",
        input_token_2_sc_account.pubkey(),
    );
    let output_token_1_sc_account = Keypair::new();
    let sig5 = create_token_account(
        &client,
        &user,
        &output_token_1_mint,
        &output_token_1_sc_account,
    )?;
    let output_token_2_sc_account = Keypair::new();
    let sig6 = create_token_account(
        &client,
        &user,
        &output_token_2_mint,
        &output_token_2_sc_account,
    )?;
    let output_token_1_user_account = Keypair::new();
    let sig7 = create_token_account(
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
    let sig8 = create_token_account(
        &client,
        &user,
        &output_token_2_mint,
        &output_token_2_user_account,
    )?;
    println!(
        "created output token 2 user account, with pubkey - {}",
        output_token_2_user_account.pubkey(),
    );

    confirm_transactions(
        &client,
        vec![sig1, sig2, sig3, sig4, sig5, sig6, sig7, sig8],
    )?;

    // Mint appropriate accounts
    let sig1 = mint_to_token_account(
        &client,
        &user,
        &input_token_1_mint,
        &input_token_1_user_account,
        100,
        input_token_decimals,
    )?;
    let balance = client.get_token_account_balance(&input_token_1_user_account.pubkey())?;
    println!(
        "created input token 1 user account, with pubkey - {}, with balance - {}",
        input_token_1_user_account.pubkey(),
        balance.amount
    );
    let sig2 = mint_to_token_account(
        &client,
        &user,
        &input_token_2_mint,
        &input_token_2_user_account,
        100,
        input_token_decimals,
    )?;
    let balance = client.get_token_account_balance(&input_token_2_user_account.pubkey())?;
    println!(
        "created input token 2 user account, with pubkey - {}, with balance - {}",
        input_token_2_user_account.pubkey(),
        balance.amount
    );
    let sig3 = mint_to_token_account(
        &client,
        &user,
        &output_token_1_mint,
        &output_token_1_sc_account,
        100,
        output_token_decimal,
    )?;
    let balance = client.get_token_account_balance(&output_token_1_sc_account.pubkey())?;
    println!(
        "created output token 1 sc account, with pubkey - {}, with balance - {}",
        output_token_1_sc_account.pubkey(),
        balance.amount
    );
    let sig4 = mint_to_token_account(
        &client,
        &user,
        &output_token_2_mint,
        &output_token_2_sc_account,
        100,
        output_token_decimal,
    )?;
    let balance = client.get_token_account_balance(&output_token_2_sc_account.pubkey())?;
    println!(
        "created output token 2 sc account, with pubkey - {}, with balance - {}",
        output_token_2_sc_account.pubkey(),
        balance.amount
    );

    confirm_transactions(&client, vec![sig1, sig2, sig3, sig4])?;

    // Initialize tokenitis
    let program_state = Keypair::new();
    let mut input_amounts = BTreeMap::new();
    input_amounts.insert(input_token_1_sc_account.pubkey(), 10);
    input_amounts.insert(input_token_2_sc_account.pubkey(), 10);
    let mut output_amounts = BTreeMap::new();
    output_amounts.insert(output_token_1_sc_account.pubkey(), 10);
    output_amounts.insert(output_token_2_sc_account.pubkey(), 10);
    let sig = initialize_tokenitis(
        &client,
        &program_state,
        &user,
        vec![
            &input_token_1_sc_account,
            &input_token_2_sc_account,
            &output_token_1_sc_account,
            &output_token_2_sc_account,
        ],
        InitializeArgs {
            input_amounts,
            output_amounts,
        },
    )?;

    confirm_transactions(&client, vec![sig])?;

    println!(
        "initialized tokenitis, with state key - {}",
        program_state.pubkey()
    );

    assert_eq!(
        100,
        client
            .get_token_account_balance(&input_token_1_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        100,
        client
            .get_token_account_balance(&input_token_2_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        0,
        client
            .get_token_account_balance(&input_token_1_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        0,
        client
            .get_token_account_balance(&input_token_2_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );

    assert_eq!(
        0,
        client
            .get_token_account_balance(&output_token_1_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        0,
        client
            .get_token_account_balance(&output_token_2_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        100,
        client
            .get_token_account_balance(&output_token_1_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        100,
        client
            .get_token_account_balance(&output_token_2_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );

    // Execute tokenitis
    let sig = execute_tokenitis(
        &client,
        &program_state,
        &user,
        vec![&input_token_1_user_account, &input_token_2_user_account],
        vec![&input_token_1_sc_account, &input_token_2_sc_account],
        vec![&output_token_1_user_account, &output_token_2_user_account],
        vec![&output_token_1_sc_account, &output_token_2_sc_account],
        ExecuteArgs {
            direction: Direction::Forward,
        },
    )?;

    confirm_transactions(&client, vec![sig])?;

    assert_eq!(
        90,
        client
            .get_token_account_balance(&input_token_1_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        90,
        client
            .get_token_account_balance(&input_token_2_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        10,
        client
            .get_token_account_balance(&input_token_1_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        10,
        client
            .get_token_account_balance(&input_token_2_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );

    assert_eq!(
        10,
        client
            .get_token_account_balance(&output_token_1_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        10,
        client
            .get_token_account_balance(&output_token_2_user_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        90,
        client
            .get_token_account_balance(&output_token_1_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );
    assert_eq!(
        90,
        client
            .get_token_account_balance(&output_token_2_sc_account.pubkey())?
            .amount
            .parse::<u64>()?
    );

    Ok(())
}

fn initialize_tokenitis(
    client: &RpcClient,
    program_state: &Keypair,
    initializer: &Keypair,
    token_accounts: Vec<&Keypair>,
    args: InitializeArgs,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let mut accounts = vec![
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new(program_state.pubkey(), false),
        AccountMeta::new_readonly(initializer.pubkey(), true),
    ];

    for acc in token_accounts {
        accounts.push(AccountMeta::new(acc.pubkey(), false))
    }

    let space = Tokenitis {
        initialized: true,
        input_amount: args.input_amounts.clone(),
        output_amount: args.output_amounts.clone(),
    }
    .try_to_vec()?
    .len();
    let rent_exemption_balance = client.get_minimum_balance_for_rent_exemption(space)?;

    let instructions = vec![
        system_instruction::create_account(
            &initializer.pubkey(),
            &program_state.pubkey(),
            rent_exemption_balance,
            space as u64,
            &tokenitis::ID,
        ),
        Instruction {
            program_id: tokenitis::ID,
            accounts,
            data: TokenitisInstructions::Initialize(args).try_to_vec()?,
        },
    ];

    let signers: Vec<&dyn Signer> = vec![program_state, initializer];
    let sig = create_and_send_tx(&client, instructions, signers, Some(&initializer.pubkey()))?;

    Ok(sig)
}

fn execute_tokenitis(
    client: &RpcClient,
    program_state: &Keypair,
    caller: &Keypair,
    caller_input_accounts: Vec<&Keypair>,
    program_input_accounts: Vec<&Keypair>,
    caller_output_accounts: Vec<&Keypair>,
    program_output_accounts: Vec<&Keypair>,
    args: ExecuteArgs,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let (pda, _nonce) = Pubkey::find_program_address(&[SEED], &tokenitis::ID);
    let mut accounts = vec![
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(program_state.pubkey(), false),
        AccountMeta::new_readonly(pda, false),
        AccountMeta::new_readonly(caller.pubkey(), true),
    ];

    for acc in [
        caller_input_accounts.as_slice(),
        program_input_accounts.as_slice(),
        caller_output_accounts.as_slice(),
        program_output_accounts.as_slice(),
    ]
    .concat()
    {
        accounts.push(AccountMeta::new(acc.pubkey(), false))
    }

    let instructions = vec![Instruction {
        program_id: tokenitis::ID,
        accounts,
        data: TokenitisInstructions::Execute(args).try_to_vec()?,
    }];

    let signers: Vec<&dyn Signer> = vec![caller];
    let sig = create_and_send_tx(&client, instructions, signers, Some(&caller.pubkey()))?;

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
    let sig = client.send_transaction(&tx)?;

    Ok(sig)
}

fn confirm_transactions(
    client: &RpcClient,
    sigs: Vec<Signature>,
) -> Result<(), Box<dyn std::error::Error>> {
    for sig in sigs {
        loop {
            if client.confirm_transaction(&sig)? {
                break;
            }
            sleep(Duration::from_millis(500))
        }
    }

    Ok(())
}
