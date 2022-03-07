use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;
pub const SEED: &[u8] = b"tokenitis";

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Tokenitis {
    pub initialized: bool,
    pub input_amount: BTreeMap<Pubkey, u64>,
    pub output_amount: BTreeMap<Pubkey, u64>,
}
