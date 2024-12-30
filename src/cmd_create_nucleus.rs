use clap::Parser;


use parity_scale_codec::{Encode, Decode};

use sp_keyring::AccountKeyring;
use ac_primitives::Config;
use substrate_api_client::{
    ac_compose_macros::{compose_extrinsic, rpc_params},
    ac_primitives::{AccountId32, DefaultRuntimeConfig, ExtrinsicSigner},
    ac_node_api::{StaticEvent},
    rpc::JsonrpseeClient,
    Api, TransactionStatus, XtStatus, SubmitAndWatch,
};

use blake2::{Blake2s256, Digest};
use sp_core::{crypto::Pair, sr25519, H256, OpaquePeerId, Bytes};

type Hash = <DefaultRuntimeConfig as Config>::Hash;

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
    // Initialize api and set the signer (sender) that is used to sign the extrinsics.
    let url = "ws://127.0.0.1:9944";
	let client = JsonrpseeClient::new(&url).await.unwrap();
	let signer = AccountKeyring::Alice.pair();

	let mut api = Api::<DefaultRuntimeConfig, _>::new(client).await.unwrap();
	let extrinsic_signer = ExtrinsicSigner::<DefaultRuntimeConfig>::new(signer);
	// Signer is needed to set the nonce and sign the extrinsic.
	api.set_signer(extrinsic_signer.clone());

	let api2 = api.clone();
    let tx = compose_extrinsic!(
        api2,
        "Nucleus",
        "create_nucleus",
        nucleus_name.as_bytes().to_vec(),
        H256::zero(),
        0,
        capacity
    ).expect("error when build extrinsic!");

    // Send and watch extrinsic until InBlock.
	let result = api
		.submit_and_watch_extrinsic_until(tx, XtStatus::Finalized)
		.await;
	println!("[+] Sent the extrinsic.");
	
	// Check if the tx really was successful:
	match result {
		Ok(report) => {
			let extrinsic_hash = report.extrinsic_hash;
			let block_hash = report.block_hash.unwrap();
			let extrinsic_status = report.status;
			let extrinsic_events = report.events.unwrap(); // Vec<RawEventDetails<Hash>

			println!("[+] Extrinsic with hash {extrinsic_hash:?} was successfully executed.",);
			println!("[+] Extrinsic got included in block with hash {block_hash:?}");
			println!("[+] Watched extrinsic until it reached the status {extrinsic_status:?}");

			assert!(matches!(extrinsic_status, TransactionStatus::Finalized(_block_hash)));

			for e in extrinsic_events {
    			println!("raw event {e:?}");
    			let ev = e.as_event::<NucleusCreatedEvent>().unwrap().unwrap();
                println!("Nucleus created.");
                println!("  id: {}", ev.id);
                println!("  name: {}", ev.name);
                println!("  capacity: {}", ev.capacity);
			}
		},
		Err(e) => {
			panic!("Expected the tx to succeed. Instead, it failed due to {e:?}");
		},
	};

    Ok(())
}
