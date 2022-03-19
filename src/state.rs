use crate::tokenitis_instruction::create_transform::CreateTransformArgs;
use crate::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;

pub const SEED: &[u8] = b"tokenitis";

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Transform {
    pub initialized: bool,
    pub metadata: TransformMetadata,
    pub inputs: BTreeMap<Pubkey, Token>,
    pub outputs: BTreeMap<Pubkey, Token>,
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
