use clap::Parser;
use subxt::utils::AccountId32;

#[derive(Debug, Clone, Parser)]
#[command(about = "List all accounts in the keystore")]
pub struct ListCmd {}

impl ListCmd {
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let home = options.get_vrx_home();
        if options.verbose {
            println!("Listing all accounts at {:?}", home);
        }
        let default_key = home.join(super::DEFAULT_KEY_FILE);
        let default_account = default_key
            .exists()
            .then(|| {
                super::read_key(&default_key)
                    .map(|keypair| AccountId32::from(keypair.public_key().0))
            })
            .transpose()
            .unwrap_or(None);
        let keys = std::fs::read_dir(home)
            .map_err(|e| anyhow::anyhow!("Couldn't read key directory, caused by {:?}", e))?;
        keys.for_each(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().unwrap().to_str().unwrap();
                    if name.starts_with("0x") {
                        if let Ok(keypair) = super::read_key(&path) {
                            let account = AccountId32::from(keypair.public_key().0);
                            if Some(&account) == default_account.as_ref() {
                                println!(
                                    "[*] {}",
                                    console::style(super::to_ss58check(account.0)).red()
                                );
                            } else {
                                println!("[ ] {}", super::to_ss58check(account.0));
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
