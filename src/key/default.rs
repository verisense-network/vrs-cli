use clap::Parser;
use sp_core::crypto::{AccountId32, Ss58Codec};
use std::io::Write;

#[derive(Debug, Clone, Parser)]
#[command(about = "Set default account for the keystore")]
pub struct DefaultCmd {
    pub account: String,
}

impl DefaultCmd {
    pub fn run(&self) -> anyhow::Result<()> {
        let home = crate::get_default_vrx_home();
        let account = AccountId32::from_ss58check(&self.account)?;
        let account: [u8; 32] = account.into();
        let pubkey = format!("0x{}", hex::encode(account));
        let key_file = home.join(pubkey);
        if !key_file.exists() {
            return Err(anyhow::anyhow!("Account not exists in the keystore"));
        }
        let private_key = std::fs::read_to_string(&key_file)?;
        let default_keypath = home.join(super::DEFAULT_KEY_FILE);
        let mut default_keyfile = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&default_keypath)?;
        default_keyfile.write_all(&private_key.as_ref())?;
        Ok(())
    }
}
