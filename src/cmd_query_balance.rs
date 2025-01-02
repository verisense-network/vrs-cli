use clap::Parser;
use std::str::FromStr;

use subxt::{OnlineClient, SubstrateConfig};
use subxt::backend::rpc::RpcClient;
use subxt::config::substrate::H256;
use subxt::utils::AccountId32;

const RPC_HOST: &str = "ws://127.0.0.1:9944";
// const RPC_HOST: &str = "wss://alpha-devnet.verisense.network";

// Generate an interface that we can use from the node's metadata.
// #[subxt::subxt(runtime_metadata_path = "metadata.scale")]
#[subxt::subxt(runtime_metadata_insecure_url = "ws://127.0.0.1:9944")]
// #[subxt::subxt(runtime_metadata_insecure_url = "wss://alpha-devnet.verisense.network")]
pub mod substrate {}

#[derive(Debug, Clone, Parser)]
#[command(name = "query-balance", about = "Query the balance of an account from the Verisense VaaS.")]
pub struct QueryBalanceCmd {
    #[arg(short = 'a', long, value_name = "SS58 string of this account")]
    account: String,
}

impl QueryBalanceCmd {
    /// Run the command
    pub fn run(&self) -> sc_cli::Result<()> {
        // Create a tokio runtime to run the async code
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        // Run the async function in the runtime
        runtime.block_on(async {
            if let Err(e) = send_to_substrate(self.account.clone()).await {
                eprintln!("Error sending to substrate: {}", e);
            }
        });

        Ok(())
    }
}

async fn send_to_substrate(
    account: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a Substrate node. Replace with your node's WebSocket URL.
    let api = OnlineClient::<SubstrateConfig>::from_url(RPC_HOST).await?;

    // The account whose balance you want to retrieve.
    // let account_id = AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let account_id = crate::utils::to_account(&account);

    let storage_query = substrate::storage().system().account(&account_id);
    // Query the balance.
    let result = api
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?;

    match result {
        Some(account_info) => {
            println!("Balance of {} is: {:?}", account_id, account_info.data.free);
        }
        None => {
            println!("Account {} not found.", account_id);
        }
    }

    Ok(())
}

