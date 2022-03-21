use crate::state::{Tokenitis, Transform};
use crate::tokenitis_instruction::execute_transform::{Direction, ExecuteTransform};

use borsh::BorshDeserialize;
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
};
use spl_token::state::Account;
use std::ops::Index;

impl ExecuteTransform<'_> {
    // Transfer funds from caller's input token accounts to smart contract
    // and retrieve funds from smart contract to caller's output token account
    pub(crate) fn execute_instruction(&mut self) -> ProgramResult {
        let accounts = &self.accounts;

        let transform_state = Transform::deserialize(&mut &**accounts.transform.data.borrow())?;
        let (transform_addr, nonce) =
            Tokenitis::find_transform_address(&self.program_id, transform_state.id);

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