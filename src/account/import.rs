use clap::Parser;
use sp_core::{crypto::AccountId32, Pair};
use std::io::Write;

#[derive(Debug, Clone, Parser)]
#[command(about = "Import an account to the keystore, support sr25519 only")]
pub struct ImportCmd {
    #[arg(long, help = "Set the imported account as the default account")]
    pub set_default: bool,
    #[arg(value_name = "SECRET")]
    pub secret: String,
}

impl ImportCmd {
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let words = self.secret.split(" ").collect::<Vec<&str>>();
        let (seed, pubkey) = if words.len() == 1 {
            let seed_hex = words[0].trim_start_matches("0x");
            let seed = hex::decode(seed_hex)
                .map_err(|_| anyhow::anyhow!("Invalid secret key: hex characters only"))?;
            let pair = sp_core::sr25519::Pair::from_seed_slice(&seed)
                .map_err(|_| anyhow::anyhow!("Invalid secret key"))?;
            let pubkey = pair.public();
            let raw_pub = pubkey.as_array_ref();
            (seed, raw_pub.to_vec())
        } else {
            let (pair, seed) =
                sp_core::sr25519::Pair::from_phrase(&self.secret, None).expect("Invalid phrase");
            let pubkey = pair.public();
            let raw_pub = pubkey.as_array_ref();
            (seed.to_vec(), raw_pub.to_vec())
        };
        let seed_hex = format!("0x{}", hex::encode(&seed));
        let pubkey_hex = format!("0x{}", hex::encode(&pubkey));
        let account = AccountId32::new(pubkey.try_into().unwrap());
        let home = options.get_vrx_home();
        if !home.exists() {
            std::fs::create_dir(&home).map_err(|_| anyhow::anyhow!("Couldn't write {:?}", home))?;
        }
        let keypath = home.join(&pubkey_hex);
        let mut keyfile = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&keypath)?;
        keyfile.write_all(&seed_hex.as_ref())?;
        let default_keypath = home.join(crate::account::DEFAULT_KEY_FILE);
        if !default_keypath.exists() {
            let mut keyfile = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&default_keypath)?;
            keyfile.write_all(&seed_hex.as_ref())?;
        } else if self.set_default {
            let mut keyfile = std::fs::OpenOptions::new()
                .write(true)
                .open(&default_keypath)?;
            keyfile.write_all(&seed_hex.as_ref())?;
        }
        println!(
            "\
                  Seed: {}\n\
            Public key: {}\n\
            Account Id: {}",
            seed_hex,
            pubkey_hex,
            super::to_ss58check(Into::<[u8; 32]>::into(account))
        );
        Ok(())
    }
}
