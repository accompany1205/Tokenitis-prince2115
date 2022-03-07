use crate::{
    execute::Execute, initialize::Initialize, instruction::Instruction,
    instruction::TokenitisInstructions,
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

// TODO client test
// TODO tokens should be identified by mint
// TODO add validation?
// TODO create gui

entrypoint!(process_instruction);
fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: &[u8],
) -> ProgramResult {
    let args = TokenitisInstructions::try_from_slice(args)?;

    let mut instruction: Box<dyn Instruction>;
    match args {
        TokenitisInstructions::Initialize(args) => {
            instruction = Box::new(Initialize::new(*program_id, accounts, args)?);
        }
        TokenitisInstructions::Execute(args) => {
            instruction = Box::new(Execute::new(*program_id, accounts, args)?);
        }
    }

    instruction.validate()?;
    instruction.execute()
}
