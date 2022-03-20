use crate::state::{Token, Tokenitis, Transform};
use crate::tokenitis_instruction::TokenitisInstruction;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::state::Account;
use std::ops::Index;

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
        let accounts = &self.accounts;

        if *accounts.token_program.key != spl_token::id() {
            msg!("invalid token program account");
            return Err(ProgramError::InvalidArgument);
        }

        let transform_state = Transform::deserialize(&mut &**accounts.transform.data.borrow())?;
        let (transform_addr, _) =
            Tokenitis::find_transform_address(&self.program_id, transform_state.id);
        if *accounts.transform.key != transform_addr {
            msg!("invalid transform account");
            return Err(ProgramError::InvalidArgument);
        }

        let mut inputs: Vec<(Pubkey, Token)> = transform_state
            .inputs
            .into_iter()
            .collect::<Vec<(Pubkey, Token)>>();
        inputs.sort();
        for i in 0..inputs.len() {
            let (mint, token) = inputs.index(i);
            let caller_input_account = accounts.caller_inputs.index(i);
            let input_account = accounts.inputs.index(i);
            if *input_account.key != token.account {
                msg!(
                    "invalid input at index - {}, unexpected program account, expected - {}, got - {}",
                    i,
                    token.account,
                    input_account.key
                );
                return Err(ProgramError::InvalidArgument);
            }

            let caller_input_account_info = Account::unpack(&**caller_input_account.data.borrow())?;
            if caller_input_account_info.mint != *mint {
                msg!("invalid input at index - {}, unexpected mint of caller_input, expected - {}, got - {}",i, mint,caller_input_account_info.mint);
                return Err(ProgramError::InvalidArgument);
            }

            if caller_input_account_info.owner != *accounts.caller.key {
                msg!("invalid input at index - {}, unexpected owner of caller_input, expected - {}, got - {}",i, accounts.caller.key,caller_input_account_info.owner);
                return Err(ProgramError::InvalidArgument);
            }
        }

        let mut outputs: Vec<(Pubkey, Token)> = transform_state
            .outputs
            .into_iter()
            .collect::<Vec<(Pubkey, Token)>>();
        outputs.sort();
        for i in 0..outputs.len() {
            let (mint, token) = outputs.index(i);
            let caller_output_account = accounts.caller_outputs.index(i);
            let output_account = accounts.outputs.index(i);
            if *output_account.key != token.account {
                msg!(
                    "invalid output at index - {}, unexpected program account, expected - {}, got - {}",
                    i,
                    token.account,
                    output_account.key
                );
                return Err(ProgramError::InvalidArgument);
            }

            let caller_output_account_info =
                Account::unpack(&**caller_output_account.data.borrow())?;
            if caller_output_account_info.mint != *mint {
                msg!("invalid output at index - {}, unexpected mint of caller_output, expected - {}, got - {}",i, mint,caller_output_account_info.mint);
                return Err(ProgramError::InvalidArgument);
            }

            if caller_output_account_info.owner != *accounts.caller.key {
                msg!("invalid output at index - {}, unexpected owner of caller_output, expected - {}, got - {}",i, accounts.caller.key,caller_output_account_info.owner);
                return Err(ProgramError::InvalidArgument);
            }
        }

        Ok(())
    }

    // Transfer funds from caller's input token accounts to smart contract
    // and retrieve funds from smart contract to caller's output token account
    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let transform_state = Transform::deserialize(&mut &**accounts.transform.data.borrow())?;
        let (transform_addr, nonce) =
            Tokenitis::find_transform_address(&self.program_id, transform_state.id.clone());

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
            let authority = accounts.transform;
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
                if authority.key.eq(&transform_addr) {
                    authority = accounts.caller;
                } else {
                    authority = accounts.transform;
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
            if !authority.key.eq(&transform_addr) {
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
                    &[&[
                        Tokenitis::transform_seed(transform_state.id).as_slice(),
                        &[nonce],
                    ]],
                )?;
            }
        }

        Ok(())
    }
}
