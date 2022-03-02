use borsh::{BorshDeserialize, BorshSerialize};
use instruction::Instruction;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use state::State;

pub struct Initialize<'a> {
    program_id: Pubkey,
    accounts: InitializeAccounts<'a>,
}

struct InitializeAccounts<'a> {
    token_program: &'a AccountInfo<'a>,
    rent: &'a AccountInfo<'a>,
    state: &'a AccountInfo<'a>,
    initializer: &'a AccountInfo<'a>,
    output_token_minter: &'a AccountInfo<'a>,
    input_token_1: &'a AccountInfo<'a>,
    input_token_2: &'a AccountInfo<'a>,
}

impl Initialize {
    pub fn new(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> Result<Self, ProgramError> {
        let accounts = &mut accounts.iter();
        Ok(Initialize {
            program_id,
            accounts: InitializeAccounts {
                token_program: next_account_info(accounts)?,
                rent: next_account_info(accounts)?,
                state: next_account_info(accounts)?,
                initializer: next_account_info(accounts)?,
                output_token_minter: next_account_info(accounts)?,
                input_token_1: next_account_info(accounts)?,
                input_token_2: next_account_info(accounts)?,
            },
        })
    }
}

impl Instruction for Initialize {
    fn validate(&self) -> ProgramResult {}

    // transfer authority of input token accounts to program
    // create the output token mint with authority as the program
    // store the state
    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let (pda, _nonce) = Pubkey::find_program_address(&[b"ctft"], &self.program_id);

        let change_authority_ix = spl_token::instruction::set_authority(
            accounts.token_program.key,
            accounts.input_token_1.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            accounts.initializer.key,
            &[&accounts.initializer.key],
        )?;

        invoke(
            &change_authority_ix,
            &[
                accounts.input_token_1.clone(),
                accounts.initializer.clone(),
                accounts.token_program.clone(),
            ],
        )?;

        let change_authority_ix = spl_token::instruction::set_authority(
            accounts.token_program.key,
            accounts.input_token_2.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            accounts.initializer.key,
            &[&accounts.initializer.key],
        )?;

        invoke(
            &change_authority_ix,
            &[
                accounts.input_token_2.clone(),
                accounts.initializer.clone(),
                accounts.token_program.clone(),
            ],
        )?;

        let create_token_ix = spl_token::instruction::initialize_mint(
            accounts.token_program.key,
            accounts.output_token_minter.key,
            &pda,
            None,
            9,
        )?;

        invoke(
            &create_token_ix,
            &[
                accounts.output_token_minter.clone(),
                accounts.rent.clone(),
                accounts.token_program.clone(),
            ],
        )?;

        let state = accounts.state.deserialize_data::<State>()?;
        if state.initialized {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let state = State {
            initialized: true,
            initializer: accounts.initializer.key.clone(),
            output_token_minter: accounts.output_token_minter.key.clone(),
            input_token_1: accounts.input_token_1.key.clone(),
            input_token_2: accounts.input_token_2.key.clone(),
        };

        accounts.state.serialize_data(&state)?;

        Ok(())
    }
}
