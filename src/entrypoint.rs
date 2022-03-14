use crate::{
    execute::Execute, initialize::Initialize, instruction::Instructions,
    instruction::TokenitisInstructions,
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

// TODO refactor
// TODO add validation?
// TODO create gui
// TODO add infinite mint
// TODO rename to transform
// TODO make state into pda and remove redundant execute arg

entrypoint!(process_instruction);
fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: &[u8],
) -> ProgramResult {
    let args = Instructions::try_from_slice(args)?;

    let mut instruction: Box<dyn TokenitisInstructions>;
    match args {
        Instructions::Initialize(args) => {
            instruction = Box::new(Initialize::new(*program_id, accounts, args)?);
        }
        Instructions::Execute(args) => {
            instruction = Box::new(Execute::new(*program_id, accounts, args)?);
        }
    }

    instruction.validate()?;
    instruction.execute()
}
