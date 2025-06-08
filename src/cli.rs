use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "vrx",
    author = "Verisense Team <dev@verisense.network>",
    about = "Command line tool for interacting with Verisense network",
    version
)]
pub struct Cli {
    #[clap(flatten)]
    pub options: Options,

    #[clap(subcommand)]
    pub cmd: SubCmd,
}

#[derive(Debug, Parser)]
pub struct Options {
    #[arg(short, long, global = true, help = "Display verbose output")]
    pub verbose: bool,

    #[arg(
        long,
        global = true,
        help = "Connect the devnet",
        default_value = "true",
        conflicts_with_all = ["rpc"]
    )]
    pub devnet: bool,

    #[arg(
        long,
        global = true,
        help = "The custom RPC endpoint to connect to. E.g. \"ws://localhost:9944\"",
        conflicts_with_all = ["devnet"]
    )]
    pub rpc: Option<String>,

    #[arg(long, global = true, help = "The vrx home path, default to \"~/.vrx\"")]
    pub vrx_dir: Option<std::path::PathBuf>,

    #[arg(
        short,
        long,
        global = true,
        help = "The private key file to use, default \"~/.vrx/default_key\""
    )]
    pub key: Option<String>,
}

pub(crate) const DEV_RPC_HOST: &'static str = "wss://rpc.beta.verisense.network";

const DEFAULT_VRX_HOME: &'static str = ".vrx";

fn get_default_vrx_home() -> std::path::PathBuf {
    if let Some(mut home) = home::home_dir() {
        home.push(DEFAULT_VRX_HOME);
        home
    } else {
        panic!("Couldn't read file under ~/.vrx")
    }
}

impl Options {
    pub(crate) fn get_rpc(&self) -> String {
        match self.rpc {
            Some(ref rpc) => rpc.clone(),
            None => {
                if self.devnet {
                    DEV_RPC_HOST.to_string()
                } else {
                    panic!("Please specify the RPC endpoint")
                }
            }
        }
    }

    pub(crate) fn get_keyfile(&self) -> std::path::PathBuf {
        match self.key {
            Some(ref key) => std::path::PathBuf::from(key),
            None => {
                let mut home = self.get_vrx_home();
                home.push(crate::account::DEFAULT_KEY_FILE);
                home
            }
        }
    }

    pub(crate) fn get_vrx_home(&self) -> std::path::PathBuf {
        match self.vrx_dir {
            Some(ref home) => home.clone(),
            None => get_default_vrx_home(),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum SubCmd {
    #[clap(subcommand)]
    Account(AccountCommand),
    #[clap(subcommand)]
    Nucleus(NucleusCommand),
    #[clap(subcommand)]
    Balance(BalanceCommand),
}

#[derive(Debug, Subcommand)]
#[command(about = "Account management subcommand")]
pub enum AccountCommand {
    Generate(crate::account::GenerateCmd),
    Inspect(crate::account::InspectCmd),
    List(crate::account::ListCmd),
    Encode(crate::account::EncodeCmd),
    SetDefault(crate::account::SetDefaultCmd),
    Import(crate::account::ImportCmd),
    Remove(crate::account::RemoveCmd),
}

impl AccountCommand {
    pub fn run(&self, options: Options) {
        let r = match self {
            AccountCommand::Generate(cmd) => cmd.run(options),
            AccountCommand::List(cmd) => cmd.run(options),
            AccountCommand::Encode(cmd) => cmd.run(),
            AccountCommand::SetDefault(cmd) => cmd.run(options),
            AccountCommand::Inspect(cmd) => cmd.run(),
            AccountCommand::Import(cmd) => cmd.run(options),
            AccountCommand::Remove(cmd) => cmd.run(options),
        };
        if let Err(e) = r {
            eprintln!("{}", e);
        }
    }
}

#[derive(Debug, Subcommand)]
#[command(about = "Nucleus management subcommand")]
pub enum NucleusCommand {
    Create(crate::nucleus::CreateNucleusCmd),
    Install(crate::nucleus::InstallCmd),
}

impl NucleusCommand {
    pub fn run(&self, options: Options) {
        let r = match self {
            NucleusCommand::Create(cmd) => cmd.run(options),
            NucleusCommand::Install(cmd) => cmd.run(options),
        };
        if let Err(e) = r {
            eprintln!("{}", e);
        }
    }
}

#[derive(Debug, Subcommand)]
#[command(about = "Balance subcommand")]
pub enum BalanceCommand {
    Query(crate::balance::QueryBalanceCmd),
    Transfer(crate::balance::TransferCmd),
}

impl BalanceCommand {
    pub fn run(&self, options: Options) {
        let r = match self {
            BalanceCommand::Transfer(cmd) => cmd.run(options),
            BalanceCommand::Query(cmd) => cmd.run(options),
        };
        if let Err(e) = r {
            eprintln!("{}", e);
        }
    }
}
