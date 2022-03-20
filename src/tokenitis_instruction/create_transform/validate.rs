use crate::state::{Token, Tokenitis};

use borsh::BorshDeserialize;

use crate::errors;
use crate::tokenitis_instruction::create_transform::CreateTransform;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::{entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey};

use spl_token::state::{Account, AccountState, Mint};

use std::ops::Index;

impl CreateTransform<'_> {
    pub(crate) fn validate_instruction(&self) -> ProgramResult {
        let args = &self.args;
        let accounts = &self.accounts;
        if args.metadata.name.len() as u64 > errors::MAX_STRING_SIZE {
            msg!(
                "transform name too large - expected - {}, got - {}",
                errors::MAX_STRING_SIZE,
                args.metadata.name.len()
            );
            return Err(ProgramError::InvalidInstructionData);
        }

        if args.metadata.image.len() as u64 > errors::MAX_STRING_SIZE {
            msg!(
                "transform image too large - expected - {}, got - {}",
                errors::MAX_STRING_SIZE,
                args.metadata.image.len()
            );
            return Err(ProgramError::InvalidInstructionData);
        }

        if *accounts.system_program.key != solana_program::system_program::id() {
            msg!("invalid system program account");
            return Err(ProgramError::InvalidArgument);
        }

        if *accounts.token_program.key != spl_token::id() {
            msg!("invalid token program account");
            return Err(ProgramError::InvalidArgument);
        }

        let (tokenitis_addr, _) = Tokenitis::find_tokenitis_address(&self.program_id);
        if *accounts.tokenitis.key != tokenitis_addr {
            msg!(
                "invalid tokenitis account, expected - {}, got - {}",
                tokenitis_addr,
                accounts.tokenitis.key
            );
            return Err(ProgramError::InvalidArgument);
        }

        let transform_num = if accounts.tokenitis.data_len() > 0 {
            Tokenitis::deserialize(&mut &**accounts.tokenitis.data.borrow())?.num_transforms
        } else {
            0
        };

        let (transform_addr, _) =
            Tokenitis::find_transform_address(&self.program_id, transform_num + 1);
        if *accounts.transform.key != transform_addr {
            msg!(
                "invalid transform account, expected - {}, got - {}",
                transform_addr,
                accounts.transform.key
            );
            return Err(ProgramError::InvalidArgument);
        }

        let mut inputs = args
            .inputs
            .clone()
            .into_iter()
            .collect::<Vec<(Pubkey, Token)>>();
        inputs.sort();
        for i in 0..args.inputs.len() {
            let (mint, token) = inputs.index(i);
            let mint_account = accounts.input_mints.index(i);
            let token_account = accounts.inputs.index(i);
            if *mint_account.key != *mint || *token_account.key != token.account {
                msg!("input information does not match at index - {}, expected - ({},{}), got - ({},{})",i,mint,token.account,mint_account.key,token_account.key);
                return Err(ProgramError::InvalidInstructionData);
            }
            let token_account_info = Account::unpack(&**token_account.data.borrow())?;
            if *mint_account.key != token_account_info.mint {
                msg!("input token account does not match mint at index - {}, token - {}, expected - {}, got - {}",i,token_account.key,token_account_info.mint,mint_account.key);
                return Err(ProgramError::InvalidAccountData);
            }

            if token_account_info.state != AccountState::Initialized {
                msg!("input token account at index - {} is not initialized", i);
                return Err(ProgramError::InvalidArgument);
            }

            if token_account_info.close_authority != COption::None {
                msg!("input token account at index - {} has a close authority", i);
                return Err(ProgramError::InvalidArgument);
            }

            if token_account_info.delegate != COption::None {
                msg!("input token account at index - {} has a delegate", i);
                return Err(ProgramError::InvalidArgument);
            }

            let mint_info = Mint::unpack(&**mint_account.data.borrow())?;
            if !mint_info.is_initialized {
                msg!("input mint at index - {} is not initialized", i);
                return Err(ProgramError::InvalidArgument);
            }
            if mint_info.freeze_authority != COption::None {
                msg!("input mint at index - {} has a freeze authority", i);
                return Err(ProgramError::InvalidArgument);
            }
        }

        let mut outputs = args
            .outputs
            .clone()
            .into_iter()
            .collect::<Vec<(Pubkey, Token)>>();
        outputs.sort();
        for i in 0..args.outputs.len() {
            let (mint, token) = outputs.index(i);
            let mint_account = accounts.output_mints.index(i);
            let token_account = accounts.outputs.index(i);
            if *mint_account.key != *mint || *token_account.key != token.account {
                msg!("output information does not match at index - {}, expected - ({},{}), got - ({},{})",i,mint,token.account,mint_account.key,token_account.key);
                return Err(ProgramError::InvalidInstructionData);
            }

            let mint_info = Mint::unpack(&**mint_account.data.borrow())?;
            if !mint_info.is_initialized {
                msg!("output mint at index - {} is not initialized", i);
                return Err(ProgramError::InvalidArgument);
            }
            if mint_info.freeze_authority != COption::None {
                msg!("output mint at index - {} has a freeze authority", i);
                return Err(ProgramError::InvalidArgument);
            }

            if mint_info.mint_authority != COption::None {
                msg!("output mint at index - {} has a mint authority", i);
                return Err(ProgramError::InvalidArgument);
            }

            let token_account_info = Account::unpack(&**token_account.data.borrow())?;
            if *mint_account.key != token_account_info.mint {
                msg!("output token account does not match mint at index - {}, token - {}, expected - {}, got - {}",i,token_account.key,token_account_info.mint,mint_account.key);
                return Err(ProgramError::InvalidAccountData);
            }

            if token_account_info.state != AccountState::Initialized {
                msg!("output token account at index - {} is not initialized", i);
                return Err(ProgramError::InvalidArgument);
            }

            if token_account_info.close_authority != COption::None {
                msg!(
                    "output token account at index - {} has a close authority",
                    i
                );
                return Err(ProgramError::InvalidArgument);
            }

            if token_account_info.delegate != COption::None {
                msg!("output token account at index - {} has a delegate", i);
                return Err(ProgramError::InvalidArgument);
            }

            if token_account_info.amount != mint_info.supply {
                msg!(
                    "output token account at index - {} does not have entire supply",
                    i
                );
                return Err(ProgramError::InvalidArgument);
            }
        }

        Ok(())
    }
}
