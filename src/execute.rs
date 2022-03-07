use crate::{
    instruction::Instruction,
    state::{Tokenitis, SEED},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::Account;
use std::ops::Index;

pub struct Execute<'a> {
    program_id: Pubkey,
    accounts: ExecuteAccounts<'a>,
    args: ExecuteArgs,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct ExecuteArgs {
    pub direction: Direction,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum Direction {
    Forward,
    Reverse,
}

struct ExecuteAccounts<'a> {
    token_program: &'a AccountInfo<'a>,
    state: &'a AccountInfo<'a>,
    pda: &'a AccountInfo<'a>,
    caller: &'a AccountInfo<'a>,
    caller_inputs: Vec<&'a AccountInfo<'a>>,
    inputs: Vec<&'a AccountInfo<'a>>,
    caller_outputs: Vec<&'a AccountInfo<'a>>,
    outputs: Vec<&'a AccountInfo<'a>>,
}

impl<'a> Execute<'a> {
    pub fn new(
        program_id: Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction: ExecuteArgs,
    ) -> Result<Self, ProgramError> {
        let accounts = &mut accounts.iter();

        let token_program = next_account_info(accounts)?;
        let state = next_account_info(accounts)?;
        let pda = next_account_info(accounts)?;
        let caller = next_account_info(accounts)?;

        let program_state = Tokenitis::deserialize(&mut &**state.data.borrow())?;

        let mut caller_inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.input_amount.len() {
            caller_inputs.push(next_account_info(accounts)?)
        }

        let mut inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.input_amount.len() {
            inputs.push(next_account_info(accounts)?)
        }

        let mut caller_outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.output_amount.len() {
            caller_outputs.push(next_account_info(accounts)?)
        }

        let mut outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.output_amount.len() {
            outputs.push(next_account_info(accounts)?)
        }

        Ok(Execute {
            program_id,
            accounts: ExecuteAccounts {
                token_program,
                state,
                pda,
                caller,
                caller_inputs,
                inputs,
                caller_outputs,
                outputs,
            },
            args: instruction,
        })
    }
}

impl Instruction for Execute<'_> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }

    // Transfer funds from caller's input token accounts to smart contract
    // and retrieve funds from smart contract to caller's output token account
    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let program_state = Tokenitis::deserialize(&mut &**accounts.state.data.borrow())?;
        let (pda, nonce) = Pubkey::find_program_address(&[SEED], &self.program_id);

        // Transfer funds from callers input token accounts to smart contract
        for i in 0..accounts.caller_inputs.len() {
            let caller_input = *accounts.caller_inputs.index(i);

            let dst: &AccountInfo;
            let amount: u64;
            if self.args.direction == Direction::Forward {
                dst = *accounts.inputs.index(i);
                let mint = Account::unpack(&**dst.data.borrow())?.mint;
                amount = *program_state
                    .input_amount
                    .get(&mint)
                    .ok_or(ProgramError::InvalidArgument)?;
            } else {
                dst = *accounts.outputs.index(i);
                let mint = Account::unpack(&**dst.data.borrow())?.mint;
                amount = *program_state
                    .output_amount
                    .get(&mint)
                    .ok_or(ProgramError::InvalidArgument)?;
            }

            let transfer_ix = spl_token::instruction::transfer(
                accounts.token_program.key,
                caller_input.key,
                dst.key,
                accounts.caller.key,
                &[accounts.caller.key],
                amount,
            )?;
            invoke(
                &transfer_ix,
                &[
                    caller_input.clone(),
                    dst.clone(),
                    accounts.caller.clone(),
                    accounts.token_program.clone(),
                ],
            )?;
        }

        // Transfer funds from smart contract to callers output token accounts
        for i in 0..accounts.caller_outputs.len() {
            let src: &AccountInfo;
            let amount: u64;
            if self.args.direction == Direction::Forward {
                src = *accounts.outputs.index(i);
                let mint = Account::unpack(&**src.data.borrow())?.mint;
                amount = *program_state
                    .output_amount
                    .get(&mint)
                    .ok_or(ProgramError::InvalidArgument)?;
            } else {
                src = *accounts.inputs.index(i);
                let mint = Account::unpack(&**src.data.borrow())?.mint;
                amount = *program_state
                    .input_amount
                    .get(&mint)
                    .ok_or(ProgramError::InvalidArgument)?;
            }

            let caller_output = *accounts.caller_outputs.index(i);
            let transfer_ix = spl_token::instruction::transfer(
                accounts.token_program.key,
                src.key,
                caller_output.key,
                &pda,
                &[&pda],
                amount,
            )?;

            invoke_signed(
                &transfer_ix,
                &[
                    src.clone(),
                    caller_output.clone(),
                    accounts.pda.clone(),
                    accounts.token_program.clone(),
                ],
                &[&[&SEED[..], &[nonce]]],
            )?;
        }

        Ok(())
    }
}
