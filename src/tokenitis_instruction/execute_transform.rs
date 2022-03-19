use crate::state::{Tokenitis, Transform, TOKENITIS_PDA};
use crate::tokenitis_instruction::TokenitisInstruction;
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
use std::collections::BTreeMap;
use std::ops::Index;

pub struct ExecuteTransform<'a> {
    program_id: Pubkey,
    accounts: ExecuteTransformAccounts<'a>,
    args: ExecuteTransformArgs,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct ExecuteTransformArgs {
    pub direction: Direction,
    pub user_inputs: BTreeMap<Pubkey, Pubkey>,
    pub user_outputs: BTreeMap<Pubkey, Pubkey>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum Direction {
    Forward,
    Reverse,
}

struct ExecuteTransformAccounts<'a> {
    token_program: &'a AccountInfo<'a>,
    tokenitis: &'a AccountInfo<'a>,
    transform: &'a AccountInfo<'a>,
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
        let tokenitis = next_account_info(accounts)?;
        let transform = next_account_info(accounts)?;
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
                tokenitis,
                transform,
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
        Ok(())
    }

    // Transfer funds from caller's input token accounts to smart contract
    // and retrieve funds from smart contract to caller's output token account
    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let transform_state = Transform::deserialize(&mut &**accounts.transform.data.borrow())?;
        let (tokenitis, nonce) = Tokenitis::find_tokenitis_address(&self.program_id);

        let mut transfer_params: Vec<(&AccountInfo, &AccountInfo, &AccountInfo, u64)> = Vec::new();
        for i in 0..accounts.caller_inputs.len() {
            let src = *accounts.caller_inputs.index(i);
            let dst = *accounts.inputs.index(i);
            let authority = accounts.caller;
            let mint = Account::unpack(&**src.data.borrow())?.mint;
            let amount = transform_state
                .inputs
                .get(&mint)
                .ok_or(ProgramError::InvalidArgument)?
                .amount;
            transfer_params.push((src, dst, authority, amount));
        }

        for i in 0..accounts.caller_outputs.len() {
            let src = *accounts.outputs.index(i);
            let dst = *accounts.caller_outputs.index(i);
            let authority = accounts.tokenitis;
            let mint = Account::unpack(&**src.data.borrow())?.mint;
            let amount = transform_state
                .outputs
                .get(&mint)
                .ok_or(ProgramError::InvalidArgument)?
                .amount;
            transfer_params.push((src, dst, authority, amount));
        }

        for (mut src, mut dst, mut authority, amount) in transfer_params {
            if self.args.direction == Direction::Reverse {
                std::mem::swap(&mut src, &mut dst);
                if authority.key.eq(&tokenitis) {
                    authority = accounts.caller;
                } else {
                    authority = accounts.tokenitis;
                }
            }

            let transfer_ix = spl_token::instruction::transfer(
                accounts.token_program.key,
                src.key,
                dst.key,
                authority.key,
                &[authority.key],
                amount,
            )?;
            if !authority.key.eq(&tokenitis) {
                invoke(
                    &transfer_ix,
                    &[
                        src.clone(),
                        dst.clone(),
                        authority.clone(),
                        accounts.token_program.clone(),
                    ],
                )?;
            } else {
                invoke_signed(
                    &transfer_ix,
                    &[
                        src.clone(),
                        dst.clone(),
                        authority.clone(),
                        accounts.token_program.clone(),
                    ],
                    &[&[TOKENITIS_PDA, &[nonce]]],
                )?;
            }
        }

        Ok(())
    }
}
