use crate::state::{Token, Tokenitis, Transform};
use crate::tokenitis_instruction::execute_transform::ExecuteTransform;

use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program::{entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey};
use spl_token::state::Account;
use std::ops::Index;

impl ExecuteTransform<'_> {
    pub(crate) fn validate_instruction(&self) -> ProgramResult {
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
}
