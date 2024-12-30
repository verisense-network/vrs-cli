use clap::Parser;
use sc_cli::{
    GenerateCmd, GenerateKeyCmdCommon, InspectKeyCmd, InspectNodeKeyCmd, SignCmd, VanityCmd,
    VerifyCmd,
};

// mod cmd_deploy;
// use cmd_deploy::DeployCmd;
mod cmd_create_nucleus;
use cmd_create_nucleus::CreateNucleusCmd;

#[derive(Debug, Parser)]
#[command(
    name = "vrx",
    author = "Verisense Team <dev@verisense.network>",
    about = "Utility for generating and restoring with Verisense chain keys",
    version
)]
#[command(before_help = r#"
 ____                      
/ ___|  ___ _ __  ___  ___ 
\___ \ / _ \ '_ \/ __|/ _ \
 ___) |  __/ | | \__ \  __/
|____/ \___|_| |_|___/\___|
"#)]
pub enum Sense {
    /// Generate a random node key, write it to a file or stdout and write the
    /// corresponding peer-id to stderr
    GenerateNodeKey(GenerateKeyCmdCommon),

    /// Generate a random account
    Generate(GenerateCmd),

    /// Gets a public key and a SS58 address from the provided Secret URI
    Inspect(InspectKeyCmd),

    /// Load a node key from a file or stdin and print the corresponding peer-id
    InspectNodeKey(InspectNodeKeyCmd),

    /// Sign a message, with a given (secret) key.
    Sign(SignCmd),

    /// Generate a seed that provides a vanity address.
    Vanity(VanityCmd),

    /// Verify a signature for a message, provided on STDIN, with a given (public or secret) key.
    Verify(VerifyCmd),

    // /// Deploy a new wasm file to verisense
    // Deploy(DeployCmd),

    /// Query the balance of an account
    CreateNucleus(CreateNucleusCmd),
}

/// Run the sense command, given the appropriate runtime.
fn main() -> Result<(), sc_cli::Error> {
    match Sense::parse() {
        Sense::GenerateNodeKey(cmd) => cmd.run(),
        Sense::Generate(cmd) => cmd.run(),
        Sense::Inspect(cmd) => cmd.run(),
        Sense::InspectNodeKey(cmd) => cmd.run(),
        Sense::Vanity(cmd) => cmd.run(),
        Sense::Verify(cmd) => cmd.run(),
        Sense::Sign(cmd) => cmd.run(),
        // Sense::Deploy(cmd) => cmd.run(),
        Sense::CreateNucleus(cmd) => cmd.run(),
    }
}
