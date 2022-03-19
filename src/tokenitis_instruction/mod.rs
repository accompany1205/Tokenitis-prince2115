use crate::tokenitis_instruction::create_transform::CreateTransformArgs;
use crate::tokenitis_instruction::execute_transform::ExecuteTransformArgs;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint::ProgramResult;

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
