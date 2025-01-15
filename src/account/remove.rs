use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(about = "Remove an account from the keystore")]
pub struct RemoveCmd {
    #[arg(help = "The account to remove")]
    pub account: String,
}

impl RemoveCmd {
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let mut home = options.get_vrx_home();
        let account = super::to_account(&self.account)?;
        let file_name = format!("0x{}", hex::encode(&account.0));
        home.push(file_name);
        if home.exists() {
            let term = console::Term::stdout();
            println!(
                "Are you sure to remove account {}? [Y/n]",
                console::style(super::to_ss58check(account.0)).red()
            );
            match term.read_key() {
                Ok(key) if key == console::Key::Char('Y') => std::fs::remove_file(&home)
                    .map_err(|e| anyhow::anyhow!("Couldn't remove key file, caused by {:?}", e)),
                _ => {
                    println!("Aborted");
                    Ok(())
                }
            }
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }
}
