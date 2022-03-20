use crate::state::{Token, TransformMetadata};
use crate::tokenitis_instruction::TokenitisInstruction;

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use std::collections::BTreeMap;

pub mod execute;
pub mod validate;

pub struct CreateTransform<'a> {
    program_id: Pubkey,
    accounts: CreateTransformAccounts<'a>,
    args: CreateTransformArgs,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct CreateTransformArgs {
    pub metadata: TransformMetadata,
    pub inputs: BTreeMap<Pubkey, Token>,
    pub outputs: BTreeMap<Pubkey, Token>,
}

// deserialize accounts instead of storing as account info
#[derive(Debug)]
struct CreateTransformAccounts<'a> {
    system_program: &'a AccountInfo<'a>,
    token_program: &'a AccountInfo<'a>,
    tokenitis: &'a AccountInfo<'a>,
    transform: &'a AccountInfo<'a>,
    creator: &'a AccountInfo<'a>,
    input_mints: Vec<&'a AccountInfo<'a>>,
    inputs: Vec<&'a AccountInfo<'a>>,
    output_mints: Vec<&'a AccountInfo<'a>>,
    outputs: Vec<&'a AccountInfo<'a>>,
}

impl<'a> CreateTransform<'a> {
    pub fn new(
        program_id: Pubkey,
        accounts: &'a [AccountInfo<'a>],
        args: CreateTransformArgs,
    ) -> Result<Self, ProgramError> {
        let accounts = &mut accounts.iter();

        let system_program = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?;
        let tokenitis = next_account_info(accounts)?;
        let transform = next_account_info(accounts)?;
        let creator = next_account_info(accounts)?;

        let mut input_mints: Vec<&AccountInfo> = Vec::new();
        for _ in 0..(args.inputs.len()) {
            input_mints.push(next_account_info(accounts)?)
        }

        let mut inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..(args.inputs.len()) {
            inputs.push(next_account_info(accounts)?)
        }

        let mut output_mints: Vec<&AccountInfo> = Vec::new();
        for _ in 0..(args.outputs.len()) {
            output_mints.push(next_account_info(accounts)?)
        }

        let mut outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..(args.outputs.len()) {
            outputs.push(next_account_info(accounts)?)
        }

        Ok(CreateTransform {
            program_id,
            accounts: CreateTransformAccounts {
                system_program,
                token_program,
                tokenitis,
                transform,
                creator,
                input_mints,
                inputs,
                output_mints,
                outputs,
            },
            args,
        })
    }
}

impl TokenitisInstruction for CreateTransform<'_> {
    fn validate(&self) -> ProgramResult {
        self.validate_instruction()
    }

    fn execute(&mut self) -> ProgramResult {
        self.execute_instruction()
    }
}
