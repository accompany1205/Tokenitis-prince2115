use crate::state::{Tokenitis};
use crate::tokenitis_instruction::create_transform::CreateTransformArgs;
use crate::tokenitis_instruction::execute_transform::ExecuteTransformArgs;
use crate::tokenitis_instruction::TokenitisInstructionType;

use crate::Result;
use borsh::BorshSerialize;

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use spl_token::instruction::{initialize_account, initialize_mint, mint_to_checked, AuthorityType};
use spl_token::state::{Account, Mint};
use std::collections::BTreeMap;

pub struct InstructionBuilder;
impl InstructionBuilder {
    pub fn create_transform_input_accounts(
        initializer: &Pubkey,
        spl_token_rent: u64,
        args: CreateTransformArgs,
    ) -> Result<Vec<Instruction>> {
        let mut instructions: Vec<Instruction> = Vec::new();
        for (mint, tok) in args.inputs.iter() {
            let program_input_account = &tok.account;
            Self::create_spl_token_account(
                mint,
                program_input_account,
                initializer,
                spl_token_rent,
            )?
            .iter()
            .for_each(|i| instructions.push(i.clone()));
        }

        Ok(instructions)
    }

    pub fn create_trarnsform_output_accounts(
        initializer: &Pubkey,
        spl_token_rent: u64,
        spl_mint_rent: u64,
        args: CreateTransformArgs,
        output_supply: BTreeMap<Pubkey, u64>,
    ) -> Result<Vec<Instruction>> {
        let mut instructions: Vec<Instruction> = Vec::new();
        for (mint, tok) in args.outputs.iter() {
            let program_output_account = &tok.account;
            Self::create_spl_token_mint(mint, initializer, None, 0, spl_mint_rent)?
                .iter()
                .for_each(|i| instructions.push(i.clone()));
            Self::create_spl_token_account(
                mint,
                program_output_account,
                initializer,
                spl_token_rent,
            )?
            .iter()
            .for_each(|i| instructions.push(i.clone()));
            let mint_entire_supply = mint_to_checked(
                &spl_token::id(),
                mint,
                program_output_account,
                initializer,
                &[initializer],
                *output_supply
                    .get(mint)
                    .ok_or(format!("could not get supply for mint - {}", mint.clone()))?,
                0,
            )?;
            let make_fixed_supply = spl_token::instruction::set_authority(
                &spl_token::id(),
                mint,
                None,
                AuthorityType::MintTokens,
                initializer,
                &[initializer],
            )?;
            instructions.push(mint_entire_supply);
            instructions.push(make_fixed_supply);
        }

        Ok(instructions)
    }

    pub fn create_transform(
        program_id: Pubkey,
        creator: &Pubkey,
        transform_num: u64,
        args: CreateTransformArgs,
    ) -> Result<Vec<Instruction>> {
        let (tokenitis, _nonce) = Tokenitis::find_tokenitis_address(&program_id);
        let (transform, _nonce) = Tokenitis::find_transform_address(&program_id, transform_num);

        let mut accounts = vec![
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new(tokenitis, false),
            AccountMeta::new(transform, false),
            AccountMeta::new_readonly(*creator, true),
        ];
        args.inputs
            .iter()
            .for_each(|(_, tok)| accounts.push(AccountMeta::new(tok.account, false)));
        args.outputs
            .iter()
            .for_each(|(_, tok)| accounts.push(AccountMeta::new(tok.account, false)));

        Ok(vec![Instruction {
            program_id,
            accounts,
            data: TokenitisInstructionType::CreateTransform(args).try_to_vec()?,
        }])
    }

    pub fn execute_transform(
        program_id: Pubkey,
        caller: &Pubkey,
        transform: &Pubkey,
        transform_state: crate::state::Transform,
        args: ExecuteTransformArgs,
    ) -> Result<Vec<Instruction>> {
        let (tokenitis, _nonce) = Tokenitis::find_tokenitis_address(&program_id);
        let mut accounts = vec![
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(tokenitis, false),
            AccountMeta::new_readonly(*transform, false),
            AccountMeta::new_readonly(*caller, true),
        ];

        let mut caller_inputs: Vec<AccountMeta> = Vec::new();
        let mut program_inputs: Vec<AccountMeta> = Vec::new();
        for (mint, tok) in transform_state.inputs.iter() {
            caller_inputs.push(AccountMeta::new(
                *args.user_inputs.get(mint).ok_or(format!(
                    "could not find caller token account for mint - {}",
                    mint.clone()
                ))?,
                false,
            ));
            program_inputs.push(AccountMeta::new(tok.account, false))
        }

        let mut caller_outputs: Vec<AccountMeta> = Vec::new();
        let mut program_outputs: Vec<AccountMeta> = Vec::new();
        for (mint, tok) in transform_state.outputs.iter() {
            caller_outputs.push(AccountMeta::new(
                *args.user_outputs.get(mint).ok_or(format!(
                    "could not find caller token account for mint - {}",
                    mint.clone()
                ))?,
                false,
            ));
            program_outputs.push(AccountMeta::new(tok.account, false))
        }

        for acc in [
            caller_inputs.as_slice(),
            program_inputs.as_slice(),
            caller_outputs.as_slice(),
            program_outputs.as_slice(),
        ]
        .concat()
        {
            accounts.push(acc)
        }

        let instructions = vec![Instruction {
            program_id,
            accounts,
            data: TokenitisInstructionType::ExecuteTransform(args).try_to_vec()?,
        }];

        Ok(instructions)
    }

    pub fn create_spl_token_mint(
        mint: &Pubkey,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        decimals: u8,
        spl_mint_rent: u64,
    ) -> Result<Vec<Instruction>> {
        let instructions = vec![
            system_instruction::create_account(
                mint_authority,
                mint,
                spl_mint_rent,
                Mint::LEN as u64,
                &spl_token::ID,
            ),
            initialize_mint(
                &spl_token::ID,
                mint,
                mint_authority,
                freeze_authority,
                decimals,
            )?,
        ];
        Ok(instructions)
    }
    pub fn create_spl_token_account(
        mint: &Pubkey,
        token_account: &Pubkey,
        authority: &Pubkey,
        rent: u64,
    ) -> Result<Vec<Instruction>> {
        let instructions = vec![
            system_instruction::create_account(
                authority,
                token_account,
                rent,
                Account::LEN as u64,
                &spl_token::ID,
            ),
            initialize_account(&spl_token::ID, token_account, mint, authority)?,
        ];
        Ok(instructions)
    }
}
