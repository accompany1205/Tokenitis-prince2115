use crate::{
    execute::Execute, initialize::Initialize, instruction::Instruction, instruction::Tokenitis,
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

// TODO upgrade dependencies
// TODO client test
// TODO create gui
// TODO add validation

entrypoint!(process_instruction);
fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: &[u8],
) -> ProgramResult {
    let args = Tokenitis::try_from_slice(args)?;

    let mut instruction: Box<dyn Instruction>;
    match args {
        Tokenitis::Initialize(args) => {
            instruction = Box::new(Initialize::new(*program_id, accounts, args)?);
        }
        Tokenitis::Execute(args) => {
            instruction = Box::new(Execute::new(*program_id, accounts, args)?);
        }
    }

    instruction.validate()?;
    instruction.execute()
}
