use crate::state::{Token, Tokenitis};
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

    pub fn create_transform_fee_accounts(
        funding_acc: &Pubkey,
        transform_creator: &Pubkey,
        args: CreateTransformArgs,
    ) -> Result<Vec<Instruction>> {
        let mut instructions: Vec<Instruction> = Vec::new();
        args.inputs.iter().for_each(|(mint, _)| {
            let ix = spl_associated_token_account::create_associated_token_account(
                funding_acc,
                transform_creator,
                mint,
            );
            instructions.push(ix);
        });

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

        let accounts = vec![
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new(tokenitis, false),
            AccountMeta::new(transform, false),
            AccountMeta::new(*creator, true),
        ];
        let mut input_args: Vec<(Pubkey, Token)> = args.inputs.clone().into_iter().collect();
        input_args.sort();
        let mut input_mints: Vec<AccountMeta> = Vec::new();
        let mut inputs: Vec<AccountMeta> = Vec::new();
        for (mint, tok) in input_args {
            input_mints.push(AccountMeta::new_readonly(mint, false));
            inputs.push(AccountMeta::new(tok.account, false))
        }

        let mut output_args: Vec<(Pubkey, Token)> = args.outputs.clone().into_iter().collect();
        output_args.sort();
        let mut output_mints: Vec<AccountMeta> = Vec::new();
        let mut outputs: Vec<AccountMeta> = Vec::new();
        for (mint, tok) in output_args {
            output_mints.push(AccountMeta::new_readonly(mint, false));
            outputs.push(AccountMeta::new(tok.account, false))
        }
        let accounts = vec![accounts, input_mints, inputs, output_mints, outputs].concat();

        Ok(vec![Instruction {
            program_id,
            accounts,
            data: TokenitisInstructionType::CreateTransform(args).try_to_vec()?,
        }])
    }

    pub fn execute_transform(
        program_id: Pubkey,
        caller: &Pubkey,
        transform_state: crate::state::Transform,
        args: ExecuteTransformArgs,
        user_inputs: BTreeMap<Pubkey, Pubkey>,
        user_outputs: BTreeMap<Pubkey, Pubkey>,
    ) -> Result<Vec<Instruction>> {
        let (transform, _nonce) =
            Tokenitis::find_transform_address(&program_id, transform_state.id);
        let mut accounts = vec![
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(transform, false),
            AccountMeta::new_readonly(*caller, true),
        ];

        let mut inputs = transform_state
            .inputs
            .into_iter()
            .collect::<Vec<(Pubkey, Token)>>();
        inputs.sort();
        let mut caller_inputs: Vec<AccountMeta> = Vec::new();
        let mut program_inputs: Vec<AccountMeta> = Vec::new();
        let mut fee_accounts: Vec<AccountMeta> = Vec::new();
        for (mint, tok) in inputs.iter() {
            caller_inputs.push(AccountMeta::new(
                *user_inputs.get(mint).ok_or(format!(
                    "could not find caller token account for mint - {}",
                    mint.clone()
                ))?,
                false,
            ));
            program_inputs.push(AccountMeta::new(tok.account, false));

            if transform_state.fee.is_some() {
                let fee_account = spl_associated_token_account::get_associated_token_address(
                    &transform_state.creator,
                    mint,
                );
                fee_accounts.push(AccountMeta::new(fee_account, false))
            }
        }

        let mut outputs = transform_state
            .outputs
            .into_iter()
            .collect::<Vec<(Pubkey, Token)>>();
        outputs.sort();
        let mut caller_outputs: Vec<AccountMeta> = Vec::new();
        let mut program_outputs: Vec<AccountMeta> = Vec::new();
        for (mint, tok) in outputs.iter() {
            caller_outputs.push(AccountMeta::new(
                *user_outputs.get(mint).ok_or(format!(
                    "could not find caller token account for mint - {}",
                    mint.clone()
                ))?,
                false,
            ));
            program_outputs.push(AccountMeta::new(tok.account, false))
        }

        for acc in vec![
            caller_inputs,
            program_inputs,
            caller_outputs,
            program_outputs,
            fee_accounts,
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
