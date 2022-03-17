extern crate borsh;
extern crate solana_program;
extern crate spl_token;

pub mod execute;
pub mod initialize;
pub mod instruction;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

solana_program::declare_id!("5fTuAJMQKJdXo5N1dZD47t4LC3VRzbfsBbpNXB6Z3XBA");
