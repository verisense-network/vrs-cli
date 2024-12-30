use clap::Parser;

use ac_primitives::Config;
use substrate_api_client::{
    ac_compose_macros::{compose_extrinsic, rpc_params},
    ac_primitives::{AccountId32, DefaultRuntimeConfig},
    Api, TransactionStatus,
};

use blake2::{Blake2s256, Digest};
use sp_core::{crypto::Pair, sr25519, H256, OpaquePeerId, Bytes};

// Define your custom event structure
#[derive(Debug, Decode)]
struct NucleusCreatedEvent {
    id: String,
    name: String,
    capacity: u16,
}

// Implement the StaticEvent trait for your custom event
impl StaticEvent for NucleusCreatedEvent {
    const PALLET: &'static str = "Nucleus";
    const EVENT: &'static str = "NucleusCreated";
}


#[derive(Debug, Clone, Parser)]
#[command(name = "create-nucleus", about = "Create a new nucleus on the Verisense VaaS.")]
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
    // Initialize api and set the signer (sender) that is used to sign the extrinsics.
    let url = "ws://127.0.0.1:9944";
	let client = JsonrpseeClient::new(&url).await.unwrap();
	let signer = AccountKeyring::Alice.pair();

	let mut api = Api::<DefaultRuntimeConfig, _>::new(client).await.unwrap();
	let extrinsic_signer = ExtrinsicSigner::<DefaultRuntimeConfig>::new(signer);
	// Signer is needed to set the nonce and sign the extrinsic.
	api.set_signer(extrinsic_signer.clone());

    let tx = compose_extrinsic!(
        api.clone(),
        "Nucleus",
        "create_nucleus",
        nucleus_name.as_bytes().to_vec(),
        H256::zero(),
        None,
        capacity,
    );

    let result = api
        .submit_and_watch_extrinsic(tx)
        .await?;
    // Get the extrinsic hash
    let tx_hash = result.extrinsic_hash;
    println!("Submitted extrinsic with hash: {:?}", tx_hash);

    // Subscribe to status updates
    let mut subscription = result.subscribe_events();

    // Monitor the subscription for status updates
    while let Some(status) = subscription.next().await {
        match status? {
            TransactionStatus::Finalized(block_hash) => {
                println!("Extrinsic finalized in block: {:?}", block_hash);

                let events = api.fetch_events_for_extrinsic(block_hash, tx_hash).await?;
                for e in events {
                    println!("raw event: {:?}", e);
                    
                    if let Some(e) = NucleusCreatedEvent::decode_from(&e) {
                        println!("Nucleus created.");
                        println!("  id: {}", ev.id);
                        println!("  name: {}", std::str::from_utf8(&ev.name).unwrap());
                        println!("  capacity: {}", ev.capacity);
                    }
                }
                
                break; // Exit the loop after finalization
            }
            _ => {
                // do nothing
            }
        }
    }

    Ok(())
}
