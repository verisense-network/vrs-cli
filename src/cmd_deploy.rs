use clap::Parser;
// use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use subxt::{OnlineClient, SubstrateConfig};

use blake2::{Blake2s256, Digest};
// use bs58;
use subxt::backend::{legacy::LegacyRpcMethods, rpc::RpcClient};
use subxt::config::substrate::{AccountId32, H256};
use subxt::config::DefaultExtrinsicParamsBuilder as Params;
use subxt::rpc_params;

const RPC_HOST: &str = "ws://127.0.0.1:9944";
// const RPC_HOST: &str = "wss://alpha-devnet.verisense.network";

// Generate an interface that we can use from the node's metadata.
// #[subxt::subxt(runtime_metadata_path = "metadata.scale")]
#[subxt::subxt(runtime_metadata_insecure_url = "ws://127.0.0.1:9944")]
// #[subxt::subxt(runtime_metadata_insecure_url = "wss://alpha-devnet.verisense.network")]
pub mod substrate {}

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
                let runtime =
                    tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

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
                        // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        // eprintln!("Deploy success!");
                    }
                });
            }

            Err(e) => eprintln!("Error reading file: {}", e),
        }

        Ok(())
    }
}

// fn calculate_digest(binary_data: &[u8]) -> String {
//     let mut hasher = Sha256::new();
//     hasher.update(binary_data);
//     let result = hasher.finalize();
//     hex::encode(result)
// }

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
    // println!("digest string: {}", digest_string);
    // Convert digest String to H256
    let digest = H256::from_slice(&hex::decode(&digest_string)?);
    println!("digest: {:?}", digest);
    // println!("digest raw: {:?}", digest.0);

    let rpc_client = RpcClient::from_url(RPC_HOST).await?;
    // Use this to construct our RPC methods:
    let rpc = LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone());
    // Create a new client
    // let api = OnlineClient::<SubstrateConfig>::new().await?;
    let api = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;

    // let signer = subxt_signer::sr25519::dev::alice();
    let signer = crate::utils::get_signer();

    // Get the peer ID via RPC
    let peer_id_str: String = rpc_client
        .request("system_localPeerId", rpc_params![])
        .await?;
    println!("Local peer ID string: {}", peer_id_str);
    let peer_id: Vec<u8> = decode_base58(&peer_id_str)?;

    // use crate::deploy_cmd::substrate::runtime_types::sp_core::H256;
    // use crate::deploy_cmd::substrate::runtime_types::sp_runtime::AccountId32;
    // Convert PeerId to OpaquePeerId
    use crate::cmd_deploy::substrate::runtime_types::sp_core::OpaquePeerId;
    // let node_id = sp_core::OpaquePeerId(peer_id_str.as_bytes().to_vec());
    let node_id = OpaquePeerId(peer_id);
    // println!("NodeId: {:?}", node_id);

    // Convert nucleus_id String to AccountId32
    let nucleus_account_id =
        AccountId32::from_str(&nucleus_id).map_err(|_| "Invalid nucleus_id format")?;
    // println!("nucleus_account_id: {:?}", nucleus_account_id);

    let current_nonce = rpc
        .system_account_next_index(&signer.public_key().into())
        .await?;
    let current_header = rpc.chain_get_header(None).await?.unwrap();
    // println!("curent_nonce: {:?}", current_nonce);
    // println!("curent_header: {:?}", current_header);

    let ext_params = Params::new()
        .mortal(&current_header, 8)
        .nonce(current_nonce)
        .build();

    // Prepare the transaction
    let tx = substrate::tx()
        .nucleus() // Replace with your actual pallet name
        .upload_nucleus_wasm(
            // Replace with your actual call name
            nucleus_account_id,
            // signer_account,
            node_id,
            digest,
        );

    // sign the transaction
    let signed_tx = api.tx().create_signed(&tx, &signer, ext_params).await?;
    let tx_bytes = signed_tx.into_encoded();
    // println!("--> signed_tx tx_bytes. {:?}", tx_bytes);

    use subxt::backend::legacy::rpc_methods::Bytes;
    // Call the nucleus_deploy RPC
    // let params = rpc_params![tx_bytes, file_content];
    let params = rpc_params![Bytes(tx_bytes), Bytes(file_content.to_vec())];
    let deploy_result: String = rpc_client.request("nucleus_deploy", params).await?;

    // Parse the result
    // let result: String = deploy_result.into_json()?;

    // Print the result
    println!("Transaction submitted: {:?}", deploy_result);

    Ok(())
}
