use clap::Parser;

use subxt::{OnlineClient, SubstrateConfig};

use subxt::backend::rpc::RpcClient;
use subxt::config::substrate::H256;
// use subxt::config::DefaultExtrinsicParamsBuilder as Params;
// use subxt::backend::legacy::rpc_methods::Bytes;
// use subxt::rpc_params;

// Generate an interface that we can use from the node's metadata.
// #[subxt::subxt(runtime_metadata_path = "metadata.scale")]
#[subxt::subxt(runtime_metadata_insecure_url = "ws://127.0.0.1:9944")]
pub mod substrate {}

#[derive(Debug, Clone, Parser)]
#[command(name = "deploy", about = "Deploy a wasm binary to the Verisense VaaS.")]
pub struct CreateNucleusCmd {
    #[arg(short = 'n', long, value_name = "name of this nucleus")]
    name: String,

    #[arg(short = 'c', long, value_name = "how many actors this nucleus wants")]
    capacity: u8,
}

impl CreateNucleusCmd {
    /// Run the command
    pub fn run(&self) -> sc_cli::Result<()> {
        // Create a tokio runtime to run the async code
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        // Run the async function in the runtime
        runtime.block_on(async {
            if let Err(e) = send_to_substrate(self.name.clone(), self.capacity).await {
                eprintln!("Error sending to substrate: {}", e);
            }
        });

        Ok(())
    }
}

async fn send_to_substrate(
    nucleus_name: String,
    capacity: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_client = RpcClient::from_url("ws://127.0.0.1:9944").await?;
    // Use this to construct our RPC methods:
    // let rpc = LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone());
    // Create a new client
    let api = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;

    // Create a signer (you'll need to replace this with actual key management)
    let signer = subxt_signer::sr25519::dev::alice();

    // Prepare the transaction
    let tx = substrate::tx()
        .nucleus() // Replace with your actual pallet name
        .create_nucleus(
            nucleus_name.as_bytes().to_vec(),
            H256::zero(),
            None,
            capacity,
        );

    let result = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, &signer)
        .await?;
    let events = result.wait_for_finalized_success().await?;
    // println!("Transaction finalized: events {:?}", events);
    for ev in events.iter().flatten() {
        if let Some(ev) = ev.as_event::<substrate::nucleus::events::NucleusCreated>()? {
            println!("Nucleus created.");
            println!("  id: {}", ev.id);
            println!("  name: {}", std::str::from_utf8(&ev.name).unwrap());
            println!("  capacity: {}", ev.capacity);
        }
    }

    Ok(())
}
