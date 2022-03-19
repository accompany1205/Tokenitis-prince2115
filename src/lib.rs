extern crate borsh;
extern crate solana_program;
extern crate spl_token;

pub mod state;
pub mod tokenitis_instruction;
pub mod util;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

solana_program::declare_id!("5fTuAJMQKJdXo5N1dZD47t4LC3VRzbfsBbpNXB6Z3XBA");
