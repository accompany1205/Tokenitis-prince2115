use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

pub fn create_pda<'a>(
    program_id: &Pubkey,
    space: usize,
    creator: &AccountInfo<'a>,
    pda: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    seed: &[u8],
) -> ProgramResult {
    let rent = solana_program::sysvar::rent::Rent::get()?
        .minimum_balance(space)
        .max(1);

    let ix = solana_program::system_instruction::create_account(
        creator.key,
        pda.key,
        rent,
        space as u64,
        program_id,
    );

    let (_, nonce) = Pubkey::find_program_address(&[seed], program_id);
    invoke_signed(
        &ix,
        &[creator.clone(), pda.clone(), system_program.clone()],
        &[&[seed, &[nonce]]],
    )
}
