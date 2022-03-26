use crate::state::Transform;
use crate::tokenitis_instruction::TokenitisInstruction;
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod execute;
pub mod validate;

pub struct ExecuteTransform<'a> {
    program_id: Pubkey,
    accounts: ExecuteTransformAccounts<'a>,
    args: ExecuteTransformArgs,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct ExecuteTransformArgs {
    pub direction: Direction,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum Direction {
    Forward,
    Reverse,
}

struct ExecuteTransformAccounts<'a> {
    token_program: &'a AccountInfo<'a>,
    transform: &'a AccountInfo<'a>,
    transform_creator: &'a AccountInfo<'a>,
    caller: &'a AccountInfo<'a>,
    caller_inputs: Vec<&'a AccountInfo<'a>>,
    inputs: Vec<&'a AccountInfo<'a>>,
    caller_outputs: Vec<&'a AccountInfo<'a>>,
    outputs: Vec<&'a AccountInfo<'a>>,
}

impl<'a> ExecuteTransform<'a> {
    pub fn new(
        program_id: Pubkey,
        accounts: &'a [AccountInfo<'a>],
        args: ExecuteTransformArgs,
    ) -> Result<Self, ProgramError> {
        let accounts = &mut accounts.iter();

        let token_program = next_account_info(accounts)?;
        let transform = next_account_info(accounts)?;
        let transform_creator = next_account_info(accounts)?;
        let caller = next_account_info(accounts)?;

        let transform_state = Transform::deserialize(&mut &**transform.data.borrow())?;

        let mut caller_inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..transform_state.inputs.len() {
            caller_inputs.push(next_account_info(accounts)?)
        }

        let mut inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..transform_state.inputs.len() {
            inputs.push(next_account_info(accounts)?)
        }

        let mut caller_outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..transform_state.outputs.len() {
            caller_outputs.push(next_account_info(accounts)?)
        }

        let mut outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..transform_state.outputs.len() {
            outputs.push(next_account_info(accounts)?)
        }

        Ok(ExecuteTransform {
            program_id,
            accounts: ExecuteTransformAccounts {
                token_program,
                transform,
                transform_creator,
                caller,
                caller_inputs,
                inputs,
                caller_outputs,
                outputs,
            },
            args,
        })
    }
}

impl TokenitisInstruction for ExecuteTransform<'_> {
    fn validate(&self) -> ProgramResult {
        self.validate_instruction()
    }

    fn execute(&mut self) -> ProgramResult {
        self.execute_instruction()
    }
}
