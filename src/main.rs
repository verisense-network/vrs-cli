mod account;
mod balance;
mod cli;
mod nucleus;

use crate::cli::*;
use clap::Parser;

fn main() -> Result<(), sc_cli::Error> {
    let cli = Cli::parse();
    match cli.cmd {
        cli::SubCmd::Nucleus(cmd) => cmd.run(cli.options),
        cli::SubCmd::Balance(cmd) => cmd.run(cli.options),
        cli::SubCmd::Account(cmd) => cmd.run(cli.options),
    }
    Ok(())
}
