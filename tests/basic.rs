extern crate solana_program;
extern crate solana_sdk;
use borsh::BorshDeserialize;

use solana_client::rpc_client::RpcClient;

use solana_program::{
    instruction::Instruction, message::Message, native_token::LAMPORTS_PER_SOL, program_pack::Pack,
    pubkey::Pubkey,
};
use solana_sdk::account::ReadableAccount;
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use spl_token::{
    instruction::mint_to_checked,
    state::{Account, Mint},
};
use std::{collections::BTreeMap, thread::sleep, time::Duration};
use tokenitis::state::{program_state_len, Token, TransformMetadata};
use tokenitis::{
    execute::{Direction, ExecuteArgs},
    initialize::InitializeArgs,
    instruction::InstructionType,
    state::Tokenitis,
};

#[test]
fn basic() -> Result<(), Box<dyn std::error::Error>> {
    let user_keypair = Keypair::new();
    let user = &user_keypair.pubkey();
    let client: RpcClient = RpcClient::new("http://localhost:8899".to_string());

    // Create account that will test the functionality of the smart contract
    let sig = client.request_airdrop(user, LAMPORTS_PER_SOL * 10)?;
    confirm_transactions(&client, vec![sig])?;
    let user_balance = client.get_balance(user)?;
    println!(
        "user account created with pubkey - {}, with balance - {}\n",
        user_keypair.pubkey(),
        user_balance
    );

    // Create input mints
    let mut instructions: Vec<Instruction> = Vec::new();
    let input_mint1 = Keypair::new();
    let input_mint2 = Keypair::new();
    let mint_rent = client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;
    InstructionType::create_spl_token_mint(&input_mint1.pubkey(), user, None, 0, mint_rent)?
        .iter()
        .for_each(|i| instructions.push(i.clone()));
    InstructionType::create_spl_token_mint(&input_mint2.pubkey(), user, None, 0, mint_rent)?
        .iter()
        .for_each(|i| instructions.push(i.clone()));
    let sig = create_and_send_tx(
        &client,
        instructions,
        vec![&user_keypair, &input_mint1, &input_mint2],
        Some(user),
    )?;
    confirm_transactions(&client, vec![sig])?;
    println!(
        "created input mints, input1 - {}, input2 - {}\n",
        input_mint1.pubkey(),
        input_mint2.pubkey()
    );

    // Initialize tokenitis
    let transform = Keypair::new();
    let output_mint1 = Keypair::new();
    let output_mint2 = Keypair::new();
    let input1_program_account = Keypair::new();
    let input2_program_account = Keypair::new();
    let output1_program_account = Keypair::new();
    let output2_program_account = Keypair::new();

    let mut inputs: BTreeMap<Pubkey, Token> = BTreeMap::new();
    let mut outputs: BTreeMap<Pubkey, Token> = BTreeMap::new();
    let mut output_supply: BTreeMap<Pubkey, u64> = BTreeMap::new();
    inputs.insert(
        input_mint1.pubkey(),
        Token {
            account: input1_program_account.pubkey(),
            amount: 10,
        },
    );
    inputs.insert(
        input_mint2.pubkey(),
        Token {
            account: input2_program_account.pubkey(),
            amount: 10,
        },
    );
    outputs.insert(
        output_mint1.pubkey(),
        Token {
            account: output1_program_account.pubkey(),
            amount: 10,
        },
    );
    outputs.insert(
        output_mint2.pubkey(),
        Token {
            account: output2_program_account.pubkey(),
            amount: 10,
        },
    );
    output_supply.insert(output_mint1.pubkey(), 100);
    output_supply.insert(output_mint2.pubkey(), 100);
    let args = InitializeArgs {
        metadata: TransformMetadata {
            name: "test123".to_string(),
            image: "".to_string(),
        },
        inputs,
        outputs,
    };
    let spl_token_rent =
        client.get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)?;
    let spl_mint_rent =
        client.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;
    let tokenitis_rent =
        client.get_minimum_balance_for_rent_exemption(program_state_len(args.clone())?)?;

    let instructions = InstructionType::create_transform_input_accounts(
        user,
        spl_token_rent,
        args.clone(),
    )?;
    let sig1 = create_and_send_tx(
        &client,
        instructions,
        vec![
            &user_keypair,
            &input1_program_account,
            &input2_program_account,
        ],
        Some(user),
    )?;

    let instructions = InstructionType::create_trarnsform_output_accounts(
        user,
        spl_token_rent,
        spl_mint_rent,
        args.clone(),
        output_supply,
    )?;
    let sig2 = create_and_send_tx(
        &client,
        instructions,
        vec![
            &user_keypair,
            &output_mint1,
            &output_mint2,
            &output1_program_account,
            &output2_program_account,
        ],
        Some(user),
    )?;
    confirm_transactions(&client, vec![sig1, sig2])?;
    println!("created program accounts");

    let instructions = tokenitis::instruction::InstructionType::initialize_tokenitis(
        user,
        &transform.pubkey(),
        tokenitis_rent,
        args.clone(),
    )?;
    let sig = create_and_send_tx(
        &client,
        instructions,
        vec![&user_keypair, &transform],
        Some(user),
    )?;
    confirm_transactions(&client, vec![sig])?;
    println!("initialized tokenitis - args - {:?}\n", args);

    let transform_account = client.get_account(&transform.pubkey())?;
    let transform_state = Tokenitis::try_from_slice(transform_account.data())?;

    // Create user token accounts
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut user_inputs: BTreeMap<Pubkey, Pubkey> = BTreeMap::new();
    let mut user_outputs: BTreeMap<Pubkey, Pubkey> = BTreeMap::new();
    let input1_user_account = Keypair::new();
    let input2_user_account = Keypair::new();
    let output1_user_account = Keypair::new();
    let output2_user_account = Keypair::new();
    user_inputs.insert(input_mint1.pubkey(), input1_user_account.pubkey());
    user_inputs.insert(input_mint2.pubkey(), input2_user_account.pubkey());
    user_outputs.insert(output_mint1.pubkey(), output1_user_account.pubkey());
    user_outputs.insert(output_mint2.pubkey(), output2_user_account.pubkey());
    let account_rent = client.get_minimum_balance_for_rent_exemption(Account::LEN)?;
    for (mint, user_account) in user_inputs.iter() {
        InstructionType::create_spl_token_account(mint, user_account, user, account_rent)?
            .iter()
            .for_each(|i| instructions.push(i.clone()));
        instructions.push(mint_to_checked(
            &spl_token::ID,
            mint,
            user_account,
            user,
            &[user],
            100,
            0,
        )?)
    }
    for (mint, user_account) in user_outputs.iter() {
        InstructionType::create_spl_token_account(mint, user_account, user, account_rent)?
            .iter()
            .for_each(|i| instructions.push(i.clone()));
    }
    let sig = create_and_send_tx(
        &client,
        instructions,
        vec![
            &user_keypair,
            &input1_user_account,
            &input2_user_account,
            &output1_user_account,
            &output2_user_account,
        ],
        Some(user),
    )?;
    confirm_transactions(&client, vec![sig])?;
    println!(
        "created user accounts, inputs - {:?}, outputs - {:?}\n",
        user_inputs.clone(),
        user_outputs.clone()
    );

    assert_eq!(
        client
            .get_token_account_balance(&input1_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );
    assert_eq!(
        client
            .get_token_account_balance(&input2_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );
    assert_eq!(
        client
            .get_token_account_balance(&output1_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&output2_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&input1_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&input2_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&output1_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );
    assert_eq!(
        client
            .get_token_account_balance(&output2_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );

    // Execute tokenitis forward
    let args = ExecuteArgs {
        direction: Direction::Forward,
        user_inputs: user_inputs.clone(),
        user_outputs: user_outputs.clone(),
    };
    let instructions = InstructionType::execute_tokenitis(
        user,
        &transform.pubkey(),
        transform_state.clone(),
        args.clone(),
    )?;
    let sig = create_and_send_tx(&client, instructions, vec![&user_keypair], Some(user))?;
    confirm_transactions(&client, vec![sig])?;

    println!(
        "successfully executed tokenitis forward, args - {:?}\n",
        args,
    );

    assert_eq!(
        client
            .get_token_account_balance(&input1_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        90
    );
    assert_eq!(
        client
            .get_token_account_balance(&input2_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        90
    );
    assert_eq!(
        client
            .get_token_account_balance(&output1_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        10
    );
    assert_eq!(
        client
            .get_token_account_balance(&output2_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        10
    );
    assert_eq!(
        client
            .get_token_account_balance(&input1_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        10
    );
    assert_eq!(
        client
            .get_token_account_balance(&input2_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        10
    );
    assert_eq!(
        client
            .get_token_account_balance(&output1_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        90
    );
    assert_eq!(
        client
            .get_token_account_balance(&output2_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        90
    );

    // Execute tokenitis reverse
    let args = ExecuteArgs {
        direction: Direction::Reverse,
        user_inputs,
        user_outputs,
    };
    let instructions = InstructionType::execute_tokenitis(
        user,
        &transform.pubkey(),
        transform_state,
        args.clone(),
    )?;
    let sig = create_and_send_tx(&client, instructions, vec![&user_keypair], Some(user))?;
    confirm_transactions(&client, vec![sig])?;

    println!(
        "successfully executed tokenitis reverse, args - {:?}\n",
        args,
    );

    assert_eq!(
        client
            .get_token_account_balance(&input1_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );
    assert_eq!(
        client
            .get_token_account_balance(&input2_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );
    assert_eq!(
        client
            .get_token_account_balance(&output1_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&output2_user_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&input1_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&input2_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        0
    );
    assert_eq!(
        client
            .get_token_account_balance(&output1_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );
    assert_eq!(
        client
            .get_token_account_balance(&output2_program_account.pubkey())?
            .amount
            .parse::<u64>()?,
        100
    );

    Ok(())
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
