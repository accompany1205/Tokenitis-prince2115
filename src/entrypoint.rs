use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::{
    execute::Execute, initialize::Initialize, instruction::Instruction, instruction::Tokenitis,
};
use bincode::config::Configuration;

// TODO compilable
// TODO upgrade dependencies
// TODO client test
// TODO create gui
// TODO add validation

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: &[u8],
) -> ProgramResult {
    let config = bincode::config::standard();
    let args = bincode::decode_from_slice::<Tokenitis, Configuration>(args, config).map_err(Err(ProgramError::))?.0;

    let mut instruction: Box<dyn Instruction>;
    match args {
        Tokenitis::Initialize(args) => {
            instruction = Initialize::new(*program_id, accounts, args)?;
        }
        Tokenitis::Execute(args) => {
            instruction = Execute::new(*program_id, accounts, args)?;
        }
    }

    instruction.validate()?;
    instruction.execute()
}
