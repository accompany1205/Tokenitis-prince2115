use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
pub const SEED: &[u8] = b"tokenitis";

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct State {
    pub initialized: bool,
    pub input_amount: HashMap<Pubkey, u64>,
    pub output_amount: HashMap<Pubkey, u64>,
}
