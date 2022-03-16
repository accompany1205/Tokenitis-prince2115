use crate::initialize::InitializeArgs;
use crate::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;

pub const SEED: &[u8] = b"tokenitis";

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Tokenitis {
    pub initialized: bool,
    pub metadata: TransformMetadata,
    pub inputs: BTreeMap<Pubkey, Token>,
    pub outputs: BTreeMap<Pubkey, Token>,
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

pub fn program_state_len(args: InitializeArgs) -> Result<usize> {
    Ok(Tokenitis {
        initialized: true,
        metadata: args.metadata.clone(),
        inputs: args.inputs.clone(),
        outputs: args.outputs.clone(),
    }
    .try_to_vec()?
    .len())
}
