use crate::state::Transform;
use crate::state::{Token, Tokenitis, TransformMetadata};
use crate::tokenitis_instruction::TokenitisInstruction;
use crate::util::create_pda;
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::AuthorityType;
use std::collections::BTreeMap;


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
    token_accounts: Vec<&'a AccountInfo<'a>>,
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

        let mut token_accounts: Vec<&AccountInfo> = Vec::new();
        for _ in 0..(args.inputs.len() + args.outputs.len()) {
            token_accounts.push(next_account_info(accounts)?)
        }

        Ok(CreateTransform {
            program_id,
            accounts: CreateTransformAccounts {
                system_program,
                token_program,
                tokenitis,
                transform,
                creator,
                token_accounts,
            },
            args,
        })
    }
}

impl TokenitisInstruction for CreateTransform<'_> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }

    // input account should be empty token account
    // output account should be an account with entire token supply
    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;

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

        for token_account in &accounts.token_accounts {
            let change_authority_ix = spl_token::instruction::set_authority(
                accounts.token_program.key,
                token_account.key,
                Some(accounts.tokenitis.key),
                AuthorityType::AccountOwner,
                accounts.creator.key,
                &[accounts.creator.key],
            )?;
            invoke(
                &change_authority_ix,
                &[
                    (*token_account).clone(),
                    accounts.creator.clone(),
                    accounts.token_program.clone(),
                ],
            )?;
        }

        let transform = Transform {
            initialized: true,
            metadata: self.args.metadata.clone(),
            inputs: self.args.inputs.clone(),
            outputs: self.args.outputs.clone(),
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

        Ok(())
    }
}
