use clap::Parser;
use std::fs::File;
use std::io::Read;

use ac_primitives::Config;
use substrate_api_client::{
    ac_compose_macros::{compose_extrinsic, rpc_params},
    ac_primitives::{AccountId32, DefaultRuntimeConfig},
    Api
};

use blake2::{Blake2s256, Digest};
use sp_core::{crypto::Pair, sr25519, H256, OpaquePeerId, Bytes};

#[derive(Debug, Clone, Parser)]
#[command(name = "deploy", about = "Deploy a wasm binary to the Verisense VaaS.")]
pub struct DeployCmd {
    #[arg(short = 'n', long, value_name = "name of this app")]
    name: String,

    #[arg(short = 'v', long, value_name = "version of this WASM file")]
    version: usize,

    #[arg(short = 'i', long, value_name = "id of this nucleus")]
    nucleus_id: String,

    #[arg(short = 'w', long, value_name = "WASM file path")]
    wasm_path: String,
}

impl DeployCmd {
    /// Run the command
    pub fn run(&self) -> sc_cli::Result<()> {
        // Handle the 'file' argument
        let mut f = File::open(&self.wasm_path)?;
        let mut file_content = Vec::new();
        match f.read_to_end(&mut file_content) {
            Ok(_) => {
                // Create a tokio runtime to run the async code
                let runtime = tokio::runtime::Runtime::new()
                    .expect("Failed to create Tokio runtime");

                // Run the async function in the runtime
                runtime.block_on(async {
                    if let Err(e) = send_to_substrate(
                        self.nucleus_id.clone(),
                        &file_content,
                        self.name.clone(),
                        self.version as u32,
                    )
                    .await
                    {
                        eprintln!("Error sending to substrate: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Error reading file: {}", e),
        }

        Ok(())
    }
}

fn calculate_blake2b_digest(binary_data: &[u8]) -> String {
    let mut hasher = Blake2s256::new();
    hasher.update(binary_data);
    let result = hasher.finalize();
    hex::encode(result)
}

fn decode_base58(encoded: &str) -> Result<Vec<u8>, bs58::decode::Error> {
    bs58::decode(encoded).into_vec()
}

async fn send_to_substrate(
    nucleus_id: String,
    file_content: &[u8],
    _name: String,
    _version: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Calculate digest
    let digest_string = calculate_blake2b_digest(file_content);
    let digest = H256::from_slice(&hex::decode(&digest_string)?);
    println!("digest: {:?}", digest);
    
    // Initialize api and set the signer (sender) that is used to sign the extrinsics.
    let url = "ws://127.0.0.1:9944";
	let client = JsonrpseeClient::new(&url).await.unwrap();
	let signer = AccountKeyring::Alice.pair();

	let mut api = Api::<DefaultRuntimeConfig, _>::new(client).await.unwrap();
	let extrinsic_signer = ExtrinsicSigner::<DefaultRuntimeConfig>::new(signer);
	// Signer is needed to set the nonce and sign the extrinsic.
	api.set_signer(extrinsic_signer.clone());
    
    // Get local peer ID
    let peer_id_str: String = api
        .get_system_local_peer_id()
        .await?;
    println!("Local peer ID string: {}", peer_id_str);
    let peer_id = decode_base58(&peer_id_str)?;
    let node_id = OpaquePeerId(peer_id);

    // Convert nucleus_id hex String to AccountId32
    // this convertion supports formats of hex or ss58
    let nucleus_account_id = nucleus_id.parse::<AccountId32>()
        .map_err(|_| "Invalid nucleus_id format")?;
    println!("Nucleus Account Id: {}", nucleus_account_id);

    // Create the extrinsic
    // Given we have set the signer in the api, so this will
    // create a new signed extrinsic
    let tx = compose_extrinsic!(
        api.clone(),
        "Nucleus",
        "upload_nucleus_wasm",
        nucleus_account_id,
        node_id,
        digest
    );

    let tx_bytes = tx.into_encoded();
    let params = rpc_params![Bytes(tx_bytes), Bytes(file_content.to_vec())]

    // make rpc request
    let deploy_result: String = api
        .client()
        .request("nucleus_deploy", params)
        .await?;
    println!("WASM deployed: {:?}", deploy_result);

    Ok(())
}
