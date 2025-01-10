use clap::Parser;
use subxt::utils::AccountId32;

#[derive(Debug, Clone, Parser)]
#[command(about = "List all accounts in the keystore")]
pub struct ListCmd {}

impl ListCmd {
    pub fn run(&self) -> anyhow::Result<()> {
        let home = crate::get_default_vrx_home();
        let default_key = home.join(super::DEFAULT_KEY_FILE);
        if default_key.exists() {
            let keypair = super::read_key(&default_key)?;
            let account = AccountId32::from(keypair.public_key().0);
            println!("[*] {}", super::to_ss58check(&account));
        }
        let keys = std::fs::read_dir(home)
            .map_err(|e| anyhow::anyhow!("Couldn't read key directory, caused by {:?}", e))?;
        keys.for_each(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().unwrap().to_str().unwrap();
                    if name.starts_with("0x") {
                        if let Ok(account) = from_pub_hex(&name) {
                            println!("[ ] {}", super::to_ss58check(&account));
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

pub(crate) fn from_pub_hex(hex: &str) -> anyhow::Result<AccountId32> {
    let hex = hex.trim_start_matches("0x");
    let bytes: Vec<u8> =
        hex::decode(hex).map_err(|_| anyhow::anyhow!("Invalid pubkey: hex characters"))?;
    let array: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid pubkey: expecting 32 bytes"))?;
    let pubkey = AccountId32(array);
    Ok(pubkey)
}
