use bip39::Mnemonic;
use clap::Parser;
use itertools::Itertools;
use sp_core::{
    crypto::{Ss58AddressFormat, Ss58Codec},
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
        short = 's',
        long = "save",
        help = "Save the generated account to the keystore"
    )]
    pub save_to_keystore: bool,
    #[arg(long, help = "Set the generated account as the default account")]
    pub set_default: bool,
}

impl GenerateCmd {
    pub fn run(&self) -> anyhow::Result<()> {
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
        let (pair, seed) =
            sp_core::sr25519::Pair::from_phrase(&phrase, None).expect("Invalid phrase");
        let prikey = format!("0x{}", hex::encode(seed.as_ref()));
        let pubkey = format!("0x{}", hex::encode(pair.public()));
        let home = crate::cli::get_default_vrx_home();
        if !home.exists() {
            std::fs::create_dir(&home).map_err(|_| anyhow::anyhow!("Couldn't write ~/.vrx"))?;
        }
        if self.save_to_keystore || self.set_default {
            let keypath = home.join(&pubkey);
            let mut keyfile = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&keypath)?;
            keyfile.write_all(&prikey.as_ref())?;
        }
        let default_keypath = home.join(crate::key::DEFAULT_KEY_FILE);
        if !default_keypath.exists() {
            let mut keyfile = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&default_keypath)?;
            keyfile.write_all(&prikey.as_ref())?;
        } else if self.set_default {
            let mut keyfile = std::fs::OpenOptions::new()
                .write(true)
                .open(&default_keypath)?;
            keyfile.write_all(&prikey.as_ref())?;
        }
        println!(
            "    Phrase: {}\n\
             Secret key: {}\n\
             Public key: {}\n\
             Account Id: {}",
            phrase,
            prikey,
            pubkey,
            pair.public().to_ss58check_with_version(prefix)
        );
        Ok(())
    }
}
