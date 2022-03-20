use crate::state::Tokenitis;
use crate::state::Transform;

use crate::util::create_pda;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::tokenitis_instruction::create_transform::CreateTransform;

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, program::invoke};
use spl_token::instruction::AuthorityType;

impl CreateTransform<'_> {
    // input account should be empty token account
    // output account should be an account with entire token supply
    pub(crate) fn execute_instruction(&mut self) -> ProgramResult {
        let accounts = &self.accounts;
        let args = self.args.clone();

        let mut tokenitis = if accounts.tokenitis.data.borrow().len() > 0 {
            Tokenitis::deserialize(&mut &**accounts.tokenitis.data.borrow())?
        } else {
            let space = Tokenitis {
                num_transforms: u64::MAX,
            }
            .try_to_vec()?
            .len();
            create_pda(
                &self.program_id,
                space,
                accounts.creator,
                accounts.tokenitis,
                accounts.system_program,
                Tokenitis::tokenitis_seed().as_slice(),
            )?;
            Tokenitis { num_transforms: 0 }
        };
        tokenitis.num_transforms += 1;
        tokenitis.serialize(&mut &mut accounts.tokenitis.data.borrow_mut()[..])?;

        let transform = Transform {
            initialized: true,
            id: tokenitis.num_transforms,
            metadata: args.metadata,
            inputs: args.inputs.into_iter().collect(),
            outputs: args.outputs.into_iter().collect(),
        };
        create_pda(
            &self.program_id,
            transform.try_to_vec()?.len(),
            accounts.creator,
            accounts.transform,
            accounts.system_program,
            Tokenitis::transform_seed(transform.id).as_slice(),
        )?;
        transform.serialize(&mut &mut accounts.transform.data.borrow_mut()[..])?;

        let token_accounts: Vec<&&AccountInfo> = accounts
            .inputs
            .iter()
            .chain(accounts.outputs.iter())
            .collect();
        for token_account in token_accounts {
            let change_authority_ix = spl_token::instruction::set_authority(
                accounts.token_program.key,
                token_account.key,
                Some(accounts.transform.key),
                AuthorityType::AccountOwner,
                accounts.creator.key,
                &[accounts.creator.key],
            )?;
            invoke(
                &change_authority_ix,
                &[
                    (**token_account).clone(),
                    accounts.creator.clone(),
                    accounts.token_program.clone(),
                ],
            )?;
        }

        Ok(())
    }
}
