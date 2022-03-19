use crate::state::{Transform, SEED};
use crate::tokenitis_instruction::create_transform::CreateTransformArgs;
use crate::tokenitis_instruction::execute_transform::ExecuteTransformArgs;
use crate::tokenitis_instruction::TokenitisInstructionType::{CreateTransform, ExecuteTransform};
use crate::{state, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use spl_token::instruction::{initialize_account, initialize_mint, mint_to_checked, AuthorityType};
use spl_token::state::{Account, Mint};
use std::collections::BTreeMap;

pub mod create_transform;
pub mod execute_transform;

pub trait TokenitisInstruction {
    fn validate(&self) -> ProgramResult;
    fn execute(&mut self) -> ProgramResult;
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub enum TokenitisInstructionType {
    CreateTransform(CreateTransformArgs),
    ExecuteTransform(ExecuteTransformArgs),
}
