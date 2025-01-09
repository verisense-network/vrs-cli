mod generate;

pub(crate) use generate::*;

use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use std::str::FromStr;
use subxt::utils::AccountId32;
use subxt_signer::sr25519::Keypair;

pub(crate) const DEFAULT_KEY_FILE: &'static str = "default_key";

pub fn read_key<P: AsRef<std::path::Path>>(file: P) -> anyhow::Result<Keypair> {
    let key =
        std::fs::read_to_string(file).map_err(|_| anyhow::anyhow!("Couldn't read private key"))?;
    let key = key.trim_start_matches("0x");
    let bytes: Vec<u8> =
        hex::decode(key).map_err(|_| anyhow::anyhow!("Invalid private key: hex characters"))?;
    let array: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid private key: expecting 32 bytes"))?;
    let pair = Keypair::from_secret_key(array)
        .map_err(|e| anyhow::anyhow!("Invalid private key: {}", e))?;
    Ok(pair)
}

pub(crate) fn to_account(account: &str) -> anyhow::Result<AccountId32> {
    AccountId32::from_str(account).map_err(|_| anyhow::anyhow!("Invalid account id"))
}

pub(crate) fn to_ss58check(account: &AccountId32) -> String {
    let prefix = Ss58AddressFormat::custom(137);
    let account = sp_core::crypto::AccountId32::from(account.0);
    account.to_ss58check_with_version(prefix)
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

pub fn list_keys() -> anyhow::Result<()> {
    let home = crate::get_default_vrx_home();
    let default_key = home.join(DEFAULT_KEY_FILE);
    if default_key.exists() {
        let keypair = read_key(&default_key)?;
        let account = AccountId32::from(keypair.public_key().0);
        println!("[*] {}", to_ss58check(&account));
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
                        println!("[ ] {}", to_ss58check(&account));
                    }
                }
            }
        }
    });
    Ok(())
}
