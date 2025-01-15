use bip39::Mnemonic;
use clap::Parser;
use itertools::Itertools;
use sp_core::{
    crypto::{AccountId32, Ss58AddressFormat, Ss58Codec},
    Pair,
};
use std::io::prelude::*;

#[derive(Debug, Clone, Parser)]
#[command(about = "Generate a random account")]
pub struct GenerateCmd {
    #[arg(short = 'w', long, value_name = "WORDS")]
    pub words: Option<usize>,
    #[arg(
        long,
        value_name = "PREFIX",
        help = "The SS58 address format prefix to use. Default is 137 used by Verisense"
    )]
    pub prefix: Option<u16>,
    #[arg(
        long = "scheme",
        help = "cryptography scheme [default: sr25519] [possible values: ed25519, sr25519]"
    )]
    pub scheme: Option<String>,
    #[arg(
        short = 's',
        long = "save",
        help = "Save the generated account to the keystore"
    )]
    pub save_to_keystore: bool,
    #[arg(long, help = "Set the generated account as the default account")]
    pub set_default: bool,
}

impl GenerateCmd {
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let words = match self.words {
            Some(words_count) if [12, 15, 18, 21, 24].contains(&words_count) => Ok(words_count),
            Some(_) => Err(anyhow::anyhow!(
                "Invalid number of words. It must be 12/15/18/21/24"
            )),
            None => Ok(12),
        }?;
        let prefix = Ss58AddressFormat::custom(self.prefix.unwrap_or(137));
        let mnemonic = Mnemonic::generate(words)
            .map_err(|e| anyhow::anyhow!("Mnemonic generation failed: {e}"))?;
        let phrase = mnemonic.words().join(" ");
        let scheme = self.scheme.as_deref().unwrap_or("sr25519");

        let (seed, pubkey) = match scheme {
            "sr25519" => {
                let (pair, seed) =
                    sp_core::sr25519::Pair::from_phrase(&phrase, None).expect("Invalid phrase");
                let pubkey = pair.public();
                let raw_pub = pubkey.as_array_ref();
                (seed.to_vec(), raw_pub.to_vec())
            }
            "ed25519" => {
                let (pair, seed) =
                    sp_core::ed25519::Pair::from_phrase(&phrase, None).expect("Invalid phrase");
                let pubkey = pair.public();
                let raw_pub = pubkey.as_array_ref();
                (seed.to_vec(), raw_pub.to_vec())
            }
            _ => return Err(anyhow::anyhow!("Invalid scheme")),
        };
        let seed_hex = format!("0x{}", hex::encode(&seed));
        let pubkey_hex = format!("0x{}", hex::encode(&pubkey));
        let home = options.get_vrx_home();
        if !home.exists() {
            std::fs::create_dir(&home).map_err(|_| anyhow::anyhow!("Couldn't write {:?}", home))?;
        }
        let account = AccountId32::new(pubkey.try_into().unwrap());
        if (self.set_default || self.save_to_keystore) && scheme != "sr25519" {
            println!("WARN: The default key could be sr25519 only, `--set-default` and `--save` are ignored.");
            println!(
                "\
                    Phrase: {}\n\
                      Seed: {}\n\
                Public key: {}\n\
                Account Id: {}",
                phrase,
                seed_hex,
                pubkey_hex,
                account.to_ss58check_with_version(prefix)
            );
        } else {
            if self.save_to_keystore || self.set_default {
                let keypath = home.join(&pubkey_hex);
                let mut keyfile = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&keypath)?;
                keyfile.write_all(&seed_hex.as_ref())?;
            }
            let default_keypath = home.join(crate::account::DEFAULT_KEY_FILE);
            if !default_keypath.exists() || self.set_default {
                let mut keyfile = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&default_keypath)?;
                keyfile.write_all(&seed_hex.as_ref())?;
            }
            println!(
                "\
                    Phrase: {}\n\
                      Seed: {}\n\
                Public key: {}\n\
                Account Id: {}",
                phrase,
                seed_hex,
                pubkey_hex,
                account.to_ss58check_with_version(prefix)
            );
        }
        Ok(())
    }
}
