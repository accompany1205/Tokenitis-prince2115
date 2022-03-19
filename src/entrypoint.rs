use crate::tokenitis_instruction::create_transform::CreateTransform;
use crate::tokenitis_instruction::execute_transform::ExecuteTransform;
use crate::tokenitis_instruction::TokenitisInstruction;
use crate::tokenitis_instruction::TokenitisInstructionType;
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);
fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: &[u8],
) -> ProgramResult {
    let instruction_type = TokenitisInstructionType::try_from_slice(args)?;

    // Can this be removed?
    let mut instruction: Box<dyn TokenitisInstruction> = match instruction_type {
        TokenitisInstructionType::CreateTransform(args) => {
            Box::new(CreateTransform::new(*program_id, accounts, args)?)
        }
        TokenitisInstructionType::ExecuteTransform(args) => {
            Box::new(ExecuteTransform::new(*program_id, accounts, args)?)
        }
    };

    instruction.validate()?;
    instruction.execute()
}
