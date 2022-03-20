use crate::state::Transform;
use crate::state::{Token, Tokenitis, TransformMetadata};
use crate::tokenitis_instruction::TokenitisInstruction;
use crate::util::create_pda;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::errors;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::AuthorityType;
use spl_token::state::{Account, AccountState, Mint};
use std::collections::BTreeMap;
use std::ops::Index;

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
        for _ in 0..(args.inputs.len()) {
            output_mints.push(next_account_info(accounts)?)
        }

        let mut outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..(args.inputs.len()) {
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
            msg!("invalid tokenitis account");
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
            msg!("invalid transform account");
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
            if mint_info.is_initialized != true {
                msg!("input mint at index - {} is not initialized", i);
                return Err(ProgramError::InvalidArgument);
            }
            if mint_info.freeze_authority != COption::None {
                msg!("input mint at index - {} has a freeze authority", i);
                return Err(ProgramError::InvalidArgument);
            }
        }

        let mut outputs = args
            .inputs
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
            if mint_info.is_initialized != true {
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

    // input account should be empty token account
    // output account should be an account with entire token supply
    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let args = self.args.clone();

        let mut tokenitis = if accounts.tokenitis.data.borrow().len() > 0 {
            Tokenitis::deserialize(&mut &**accounts.tokenitis.data.borrow())?
        } else {
            let space = Tokenitis {
                num_transforms: u64::MAX,
            }
            .try_to_vec()?
            .len();
            create_pda(
                &self.program_id,
                space,
                accounts.creator,
                accounts.tokenitis,
                accounts.system_program,
                Tokenitis::tokenitis_seed().as_slice(),
            )?;
            Tokenitis { num_transforms: 0 }
        };
        tokenitis.num_transforms += 1;
        tokenitis.serialize(&mut &mut accounts.tokenitis.data.borrow_mut()[..])?;

        let transform = Transform {
            initialized: true,
            id: tokenitis.num_transforms.clone(),
            metadata: args.metadata,
            inputs: args.inputs.into_iter().collect(),
            outputs: args.outputs.into_iter().collect(),
        };
        create_pda(
            &self.program_id,
            transform.try_to_vec()?.len(),
            accounts.creator,
            accounts.transform,
            accounts.system_program,
            Tokenitis::transform_seed(tokenitis.num_transforms).as_slice(),
        )?;
        transform.serialize(&mut &mut accounts.transform.data.borrow_mut()[..])?;

        let token_accounts: Vec<&&AccountInfo> = accounts
            .inputs
            .iter()
            .chain(accounts.outputs.iter())
            .collect();
        for token_account in token_accounts {
            let change_authority_ix = spl_token::instruction::set_authority(
                accounts.token_program.key,
                token_account.key,
                Some(accounts.transform.key),
                AuthorityType::AccountOwner,
                accounts.creator.key,
                &[accounts.creator.key],
            )?;
            invoke(
                &change_authority_ix,
                &[
                    (**token_account).clone(),
                    accounts.creator.clone(),
                    accounts.token_program.clone(),
                ],
            )?;
        }

        Ok(())
    }
}
