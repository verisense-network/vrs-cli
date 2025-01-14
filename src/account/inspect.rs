use clap::Parser;
use sp_core::{
    crypto::{AccountId32, Ss58AddressFormat, Ss58Codec},
    Pair,
};

#[derive(Debug, Clone, Parser)]
#[command(about = "Gets a public key and an account id from the phrase or secret seed")]
pub struct InspectCmd {
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
    #[arg(value_name = "SECRET")]
    pub secret: String,
}

impl InspectCmd {
    pub fn run(&self) -> anyhow::Result<()> {
        let prefix = Ss58AddressFormat::custom(self.prefix.unwrap_or(137));
        let scheme = self.scheme.as_deref().unwrap_or("sr25519");
        let words = self.secret.split(" ").collect::<Vec<&str>>();
        let (seed, pubkey) = if words.len() == 1 {
            let pri_hex = words[0].trim_start_matches("0x");
            let pri = hex::decode(pri_hex)
                .map_err(|_| anyhow::anyhow!("Invalid secret key: hex characters only"))?;
            match scheme {
                "sr25519" => {
                    let pair = sp_core::sr25519::Pair::from_seed_slice(&pri)
                        .map_err(|_| anyhow::anyhow!("Invalid secret key"))?;
                    let pubkey = pair.public();
                    let raw_pub = pubkey.as_array_ref();
                    (pri, raw_pub.to_vec())
                }
                "ed25519" => {
                    let pair = sp_core::ed25519::Pair::from_seed_slice(&pri)
                        .map_err(|_| anyhow::anyhow!("Invalid secret key"))?;
                    let pubkey = pair.public();
                    let raw_pub = pubkey.as_array_ref();
                    (pri.to_vec(), raw_pub.to_vec())
                }
                _ => return Err(anyhow::anyhow!("Invalid scheme")),
            }
        } else {
            match scheme {
                "sr25519" => {
                    let (pair, seed) = sp_core::sr25519::Pair::from_phrase(&self.secret, None)
                        .expect("Invalid phrase");
                    let pubkey = pair.public();
                    let raw_pub = pubkey.as_array_ref();
                    (seed.to_vec(), raw_pub.to_vec())
                }
                "ed25519" => {
                    let (pair, seed) = sp_core::ed25519::Pair::from_phrase(&self.secret, None)
                        .expect("Invalid phrase");
                    let pubkey = pair.public();
                    let raw_pub = pubkey.as_array_ref();
                    (seed.to_vec(), raw_pub.to_vec())
                }
                _ => return Err(anyhow::anyhow!("Invalid scheme")),
            }
        };
        let seed_hex = format!("0x{}", hex::encode(&seed));
        let pubkey_hex = format!("0x{}", hex::encode(&pubkey));
        let account = AccountId32::new(pubkey.try_into().unwrap());
        println!(
            "\
                  Seed: {}\n\
            Public key: {}\n\
            Account Id: {}",
            seed_hex,
            pubkey_hex,
            account.to_ss58check_with_version(prefix)
        );
        Ok(())
    }
}
