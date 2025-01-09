mod balance;
mod cli;
mod key;
mod nucleus;

use crate::cli::*;
use clap::Parser;

fn main() -> Result<(), sc_cli::Error> {
    let cli = Cli::parse();
    match cli.cmd {
        cli::SubCmd::Nucleus(cmd) => cmd.run(cli.options),
        cli::SubCmd::Balance(cmd) => cmd.run(cli.options),
        cli::SubCmd::Key(cmd) => cmd.run(),
    }
    Ok(())
}
