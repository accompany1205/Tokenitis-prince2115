use bincode::Decode;
use bincode::Encode;
use instruction::Instruction;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use spl_token::solana_program::program_pack::Pack;
use state::{InputConfig, OutputConfig, State};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::{Deref, Index};

pub struct GetInput<'a> {
    program_id: Pubkey,
    accounts: GetInputAccounts<'a>,
    instruction: GetInputInstruction,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct GetInputInstruction {}

struct GetInputAccounts<'a> {
    token_program: &'a AccountInfo<'a>,
    state: &'a AccountInfo<'a>,
    caller: &'a AccountInfo<'a>,
    caller_inputs: Vec<&'a AccountInfo<'a>>,
    inputs: Vec<&'a AccountInfo<'a>>,
    caller_outputs: Vec<&'a AccountInfo<'a>>,
    outputs: Vec<&'a AccountInfo<'a>>,
}

// TODO add validation

impl GetInput {
    pub fn new(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> Result<Self, ProgramError> {
        let accounts = &mut accounts.iter();

        let token_program = next_account_info(accounts)?;
        let state = next_account_info(accounts)?;
        let caller = next_account_info(accounts)?;

        let program_state = state.deserialize_data::<State>()?;

        let mut caller_inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.input_configs.len() {
            caller_inputs.push(next_account_info(accounts)?)
        }

        let mut inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.input_configs.len() {
            inputs.push(next_account_info(accounts)?)
        }

        let mut caller_outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.output_configs.len() {
            caller_outputs.push(next_account_info(accounts)?)
        }

        let mut outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..program_state.output_configs.len() {
            outputs.push(next_account_info(accounts)?)
        }

        Ok(GetInput {
            program_id,
            accounts: GetInputAccounts {
                token_program,
                state,
                caller,
                caller_inputs,
                inputs,
                caller_outputs,
                outputs,
            },
            instruction: GetInputInstruction {},
        })
    }
}

impl Instruction for GetInput {
    fn validate(&self) -> ProgramResult {}

    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let program_state = accounts.state.deserialize_data::<State>()?;
        let (pda, _nonce) = Pubkey::find_program_address(&[b"tokenitis"], &self.program_id);

        for i in 0..accounts.caller_inputs.len() {
            let caller_input = *accounts.caller_inputs.index(i);
            let output = *accounts.outputs.index(i);
            let transfer_ix = spl_token::instruction::transfer(
                accounts.token_program.key,
                caller_input.key,
                output.key,
                accounts.caller.key,
                &[accounts.caller.key],
                program_state
                    .input_configs
                    .get(output.key)
                    .ok_or(ProgramError::InvalidArgument)?
                    .amount,
            )?;

            invoke(
                &transfer_ix,
                &[
                    caller_input.clone(),
                    output.clone(),
                    accounts.caller.clone(),
                    accounts.token_program.clone(),
                ],
            )?;
        }

        for i in 0..accounts.caller_outputs.len() {
            let input = *accounts.inputs.index(i);
            let caller_output = *accounts.caller_outputs.index(i);
            let transfer_ix = spl_token::instruction::transfer(
                accounts.token_program.key,
                input.key,
                caller_output.key,
                &pda,
                &[&pda],
                program_state
                    .input_configs
                    .get(input.key)
                    .ok_or(ProgramError::InvalidArgument)?
                    .amount,
            )?;

            invoke_signed(
                &transfer_ix,
                &[
                    input.clone(),
                    caller_output.clone(),
                    pda.clone(),
                    accounts.token_program.clone(),
                ],
                &[&[b"tokenitis"]],
            )?;
        }

        Ok(())
    }
}
