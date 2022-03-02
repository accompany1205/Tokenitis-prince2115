use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

use crate::instruction::EscrowInstruction;
use crate::processor::Processor;

// TODO implement combine to token

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // TODO remove existing instructions
    // TODO this should be a trait - instruction should be validated then executed

    match instruction {
        EscrowInstruction::InitEscrow { amount } => {
            msg!("Instruction: InitEscrow");
            Processor::process_init_escrow(accounts, amount, program_id)
        }
        EscrowInstruction::Exchange { amount } => {
            msg!("Instruction: Exchange");
            Processor::process_exchange(accounts, amount, program_id)
        }
    }
}
