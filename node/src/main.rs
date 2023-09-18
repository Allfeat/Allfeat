//! Allfeat CLI library.
#![warn(missing_docs)]

mod chain_specs;
#[macro_use]
mod service;
mod cli;
mod client;
mod command;
mod command_helper;
mod eth;
mod rpc;

fn main() -> sc_cli::Result<()> {
	command::run()
}
