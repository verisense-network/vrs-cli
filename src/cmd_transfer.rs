use clap::Parser;
use std::str::FromStr;

use subxt::{OnlineClient, SubstrateConfig};
use subxt::backend::rpc::RpcClient;
use subxt::config::substrate::H256;
use subxt::utils::{AccountId32, MultiAddress};

const RPC_HOST: &str = "ws://127.0.0.1:9944";
// const RPC_HOST: &str = "wss://alpha-devnet.verisense.network";

// Generate an interface that we can use from the node's metadata.
// #[subxt::subxt(runtime_metadata_path = "metadata.scale")]
#[subxt::subxt(runtime_metadata_insecure_url = "ws://127.0.0.1:9944")]
// #[subxt::subxt(runtime_metadata_insecure_url = "wss://alpha-devnet.verisense.network")]
pub mod substrate {}

#[derive(Debug, Clone, Parser)]
#[command(name = "transfer", about = "Transfer some amount of token to another account on Verisense VaaS.")]
pub struct TransferCmd {
    #[arg(short = 'a', long, value_name = "SS58 string of the destination account")]
    account: String,
    #[arg(short = 'n', long, value_name = "token amount")]
    num: u128,
}

impl TransferCmd {
    /// Run the command
    pub fn run(&self) -> sc_cli::Result<()> {
        // Create a tokio runtime to run the async code
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        // Run the async function in the runtime
        runtime.block_on(async {
            if let Err(e) = send_to_substrate(self.account.clone(), self.num).await {
                eprintln!("Error sending to substrate: {}", e);
            }
        });

        Ok(())
    }
}

async fn send_to_substrate(
    account: String,
    num: u128
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a Substrate node. Replace with your node's WebSocket URL.
    let api = OnlineClient::<SubstrateConfig>::from_url(RPC_HOST).await?;

    let from_account = crate::utils::get_signer();
    let to_account = crate::utils::to_account(&account);

    let balance_transfer_tx = substrate::tx().balances().transfer_allow_death(MultiAddress::Id(to_account), num);

    // Submit the balance transfer extrinsic from the signer, and wait for it to be successful
    // and in a finalized block. We get back the extrinsic events if all is well.
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&balance_transfer_tx, &from_account)
        .await?
        .wait_for_finalized_success()
        .await?;

    // Find a Transfer event and print it.
    let transfer_event = events.find_first::<substrate::balances::events::Transfer>()?;
    if let Some(event) = transfer_event {
        println!("Balance transfer success: {event:?}");
    }

    Ok(())
}

