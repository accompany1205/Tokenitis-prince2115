use crate::tokenitis_instruction::create_transform::CreateTransformArgs;
use crate::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;

// pda seed for the account that stores global state
pub const TOKENITIS_PDA: &[u8] = b"tokenitis";
pub const TRANSFORM_PREFIX: &str = "transform";

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Tokenitis {
    pub num_transforms: u64,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Transform {
    pub initialized: bool,
    pub metadata: TransformMetadata,
    pub inputs: BTreeMap<Pubkey, Token>,
    pub outputs: BTreeMap<Pubkey, Token>,
}

impl Tokenitis {
    pub fn find_tokenitis_address(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[TOKENITIS_PDA], program_id)
    }
    pub fn find_transform_address(program_id: &Pubkey, transform_num: u64) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[format!("{}-{}", TRANSFORM_PREFIX, transform_num).as_bytes()],
            program_id,
        )
    }
}

impl Transform {
    pub fn transform_len(args: CreateTransformArgs) -> Result<usize> {
        Ok(Transform {
            initialized: true,
            metadata: args.metadata.clone(),
            inputs: args.inputs.clone(),
            outputs: args.outputs,
        }
        .try_to_vec()?
        .len())
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct TransformMetadata {
    pub name: String,
    pub image: String,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Token {
    pub account: Pubkey,
    pub amount: u64,
}
