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
        SubCmd::Nucleus(cmd) => cmd.run(cli.options),
        SubCmd::Balance(cmd) => cmd.run(cli.options),
        SubCmd::Account(cmd) => cmd.run(cli.options),
    }
}
