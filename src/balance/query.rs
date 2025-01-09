use clap::Parser;
use subxt::utils::AccountId32;
use subxt::{OnlineClient, SubstrateConfig};

#[derive(Debug, Clone, Parser)]
#[command(about = "Query the balance of an account")]
pub struct QueryBalanceCmd {
    #[arg(
        long,
        value_name = "account",
        help = "Query balance of the account or the default account if not specified"
    )]
    account: Option<String>,
}

impl QueryBalanceCmd {
    /// Run the command
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let rpc = options.get_rpc();
        let key_file = options.get_keyfile();
        let account = match self.account {
            Some(ref account) => crate::key::to_account(account)?,
            None => {
                let signer = crate::key::read_key(key_file)?;
                signer.public_key().into()
            }
        };
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async { query(rpc, account).await })
    }
}

async fn query(rpc: impl AsRef<str>, account: AccountId32) -> anyhow::Result<()> {
    let api = OnlineClient::<SubstrateConfig>::from_url(rpc).await?;
    let storage_query = vrs_metadata::codegen::storage().system().account(&account);
    let result = api
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?;
    let info = result
        .map(|info| (info.data.free, info.data.reserved))
        .unwrap_or_default();
    println!("Free: {}, Reserved: {}", info.0, info.1);
    Ok(())
}
