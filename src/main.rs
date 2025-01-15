mod account;
mod balance;
mod cli;
mod nucleus;
mod runtime;

use crate::cli::*;
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        cli::SubCmd::Nucleus(cmd) => cmd.run(cli.options),
        cli::SubCmd::Balance(cmd) => cmd.run(cli.options),
        cli::SubCmd::Account(cmd) => cmd.run(cli.options),
    }
}
