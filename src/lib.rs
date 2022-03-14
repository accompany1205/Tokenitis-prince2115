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

solana_program::declare_id!("6PkfvXv83Jnha5wtZ3eYMzsxn2JFWwjKqzJY8Pw4y9pm");
