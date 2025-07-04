use clap::Parser;
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use std::str::FromStr;

#[derive(Debug, Clone, Parser)]
#[command(about = "Gets an account id from the public key")]
pub struct EncodeCmd {
    #[arg(
        long,
        value_name = "PREFIX",
        help = "The SS58 address format prefix to use. Default is 137 used by Verisense"
    )]
    pub prefix: Option<u16>,
    #[arg(
        value_name = "PUBKEY",
        help = "Transform a PUBKEY or a SS58 address to an account id"
    )]
    pub public: String,
}

impl EncodeCmd {
    pub fn run(&self) -> anyhow::Result<()> {
        let prefix = Ss58AddressFormat::custom(self.prefix.unwrap_or(137));
        if self.public.starts_with("0x") {
            let pub_hex = self.public.trim_start_matches("0x");
            let pubkey = hex::decode(pub_hex)
                .map_err(|_| anyhow::anyhow!("Invalid public key: hex characters only"))?;
            let account = AccountId32::new(
                pubkey
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid public key: 32 bytes required"))?,
            );
            println!("{}", account.to_ss58check_with_version(prefix));
        } else {
            let account = AccountId32::from_str(&self.public)
                .map_err(|_| anyhow::anyhow!("Invalid public key or SS58 address"))?;
            println!("{}", account.to_ss58check_with_version(prefix));
        }
        Ok(())
    }
}
