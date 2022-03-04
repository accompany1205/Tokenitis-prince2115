use crate::{
    instruction::Instruction,
    state::{State, SEED},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub struct Execute<'a> {
    program_id: Pubkey,
    accounts: ExecuteAccounts<'a>,
    args: ExecuteArgs,
}

pub struct ExecuteArgs {
    direction: Direction,
}

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

impl Execute<'_> {
    pub fn new(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        instruction: ExecuteArgs,
    ) -> Result<Box<dyn Instruction>, ProgramError> {
        let accounts = &mut accounts.iter();

        let token_program = next_account_info(accounts)?;
        let state = next_account_info(accounts)?;
        let pda = next_account_info(accounts)?;
        let caller = next_account_info(accounts)?;

        let program_state = state.deserialize_data::<State>()?;

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

        Ok(Box::new(Execute {
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
        }))
    }
}

impl Instruction for Execute<'_> {
    fn validate(&self) -> ProgramResult {}

    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let program_state = accounts.state.deserialize_data::<State>()?;
        let (pda, _nonce) = Pubkey::find_program_address(&[SEED], &self.program_id);

        for i in 0..accounts.caller_inputs.len() {
            let caller_input = *accounts.caller_inputs.index(i);

            let dst: &AccountInfo;
            let mut amount: u64 = 0;
            if self.args.direction == Direction::Forward {
                dst = *accounts.inputs.index(i);
                amount = *program_state
                    .input_amount
                    .get(dst.key)
                    .ok_or(ProgramError::InvalidArgument)?;
            } else {
                dst = *accounts.outputs.index(i);
                amount = *program_state
                    .output_amount
                    .get(dst.key)
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

        for i in 0..accounts.caller_outputs.len() {
            let src: &AccountInfo;
            let amount: u64;
            if self.args.direction == Direction::Forward {
                src = *accounts.outputs.index(i);
                amount = *program_state
                    .output_amount
                    .get(src.key)
                    .ok_or(ProgramError::InvalidArgument)?;
            } else {
                src = *accounts.inputs.index(i);
                amount = *program_state
                    .input_amount
                    .get(src.key)
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
                &[&[SEED]],
            )?;
        }

        Ok(())
    }
}
