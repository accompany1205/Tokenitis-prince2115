use bincode::Decode;
use bincode::Encode;
use instruction::Instruction;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use spl_token::solana_program::program_pack::Pack;
use state::{InputConfig, OutputConfig, State};
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct Initialize<'a> {
    program_id: Pubkey,
    accounts: InitializeAccounts<'a>,
    instruction: InitializeInstruction,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct InitializeInstruction {
    num_inputs: u64,
    num_outputs: u64,
    input_configs: HashMap<Pubkey, InputConfig>,
    output_configs: HashMap<Pubkey, OutputConfig>,
}

struct InitializeAccounts<'a> {
    token_program: &'a AccountInfo<'a>,
    rent: &'a AccountInfo<'a>,
    state: &'a AccountInfo<'a>,
    initializer: &'a AccountInfo<'a>,
    inputs: Vec<&'a AccountInfo<'a>>,
    outputs: Vec<&'a AccountInfo<'a>>,
}

// TODO simplify
// TODO client test
// TODO create gui
// TODO add validation

impl Initialize {
    pub fn new(
        program_id: Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> Result<Self, ProgramError> {
        let instruction = bincode::deserialize::<InitializeInstruction>(instruction_data)?;
        let accounts = &mut accounts.iter();

        let token_program = next_account_info(accounts)?;
        let rent = next_account_info(accounts)?;
        let state = next_account_info(accounts)?;
        let initializer = next_account_info(accounts)?;

        let mut inputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..instruction.num_inputs {
            inputs.push(next_account_info(accounts)?)
        }

        let mut outputs: Vec<&AccountInfo> = Vec::new();
        for _ in 0..instruction.num_outputs {
            outputs.push(next_account_info(accounts)?)
        }

        Ok(Initialize {
            program_id,
            accounts: InitializeAccounts {
                token_program,
                rent,
                state,
                initializer,
                inputs,
                outputs,
            },
            instruction,
        })
    }
}

impl Instruction for Initialize {
    fn validate(&self) -> ProgramResult {}

    // input account should be empty token account
    // output account should be an account with entire token supply
    fn execute(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let (pda, _nonce) = Pubkey::find_program_address(&[b"tokenitis"], &self.program_id);

        let mut mint_to_account: HashMap<Pubkey, Pubkey> = HashMap::new();
        for acc in accounts.inputs.iter().chain(accounts.outputs.iter()) {
            let acc_data = spl_token::state::Account::unpack(acc.data.borrow())?;
            mint_to_account.insert(acc_data.mint, acc.key.clone());
            let change_authority_ix = spl_token::instruction::set_authority(
                accounts.token_program.key,
                acc.key,
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
        }

        let state = accounts.state.deserialize_data::<State>()?;
        if state.initialized {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let state = State {
            initialized: true,
            initializer: accounts.initializer.key.clone(),
            mint_to_account,
            input_configs: self.instruction.input_configs.clone(),
            output_configs: self.instruction.output_configs.clone(),
        };

        accounts.state.serialize_data(&state)?;

        Ok(())
    }
}
