use blake2::{Blake2s256, Digest};
use clap::Parser;
use std::fs::File;
use std::io::Read;
use subxt::backend::{legacy::LegacyRpcMethods, rpc::RpcClient};
use subxt::config::substrate::{AccountId32, H256};
use subxt::config::DefaultExtrinsicParamsBuilder as Params;
use subxt::rpc_params;
use subxt::{OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::Keypair;
use vrs_metadata::codegen::runtime_types::sp_core::OpaquePeerId;

#[derive(Debug, Clone, Parser)]
#[command(name = "install", about = "Install WASM code into the associate ID")]
pub struct InstallCmd {
    #[arg(
        long = "id",
        value_name = "ID",
        help = "The nucleus ID to install WASM"
    )]
    nucleus_id: String,

    #[arg(long = "wasm", value_name = "WASM", help = "The path to the WASM file")]
    wasm_path: String,
}

impl InstallCmd {
    /// Run the command
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let rpc = options.get_rpc();
        let key_file = options.get_keyfile();
        let signer = crate::account::read_key(key_file)?;
        let mut f = File::open(&self.wasm_path)
            .map_err(|_| anyhow::anyhow!("Unable to read WASM from {}", self.wasm_path))?;
        let mut file_content = Vec::new();
        let id = crate::account::to_account(&self.nucleus_id)?;
        f.read_to_end(&mut file_content)
            .map_err(|e| anyhow::anyhow!("Error occur while reading WASM file: {}", e))?;
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async { install(rpc, signer, id, &file_content, options.verbose).await })
    }
}

fn calculate_blake2b_digest(binary_data: &[u8]) -> H256 {
    let mut hasher = Blake2s256::new();
    hasher.update(binary_data);
    let result = hasher.finalize();
    H256::from_slice(&result)
}

fn decode_base58(encoded: &str) -> Result<Vec<u8>, bs58::decode::Error> {
    bs58::decode(encoded).into_vec()
}

async fn install(
    rpc_url: impl AsRef<str>,
    signer: Keypair,
    nucleus_id: AccountId32,
    file_content: &[u8],
    verbose: bool,
) -> anyhow::Result<()> {
    let digest = calculate_blake2b_digest(file_content);
    if verbose {
        println!("WASM digest: {:?}", digest);
    }
    let rpc_client = RpcClient::from_url(rpc_url).await?;
    let rpc = LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone());
    let peer_id_str: String = rpc_client
        .request("system_localPeerId", rpc_params![])
        .await?;
    if verbose {
        println!("Uploading WASM to node {}", peer_id_str);
    }
    let api = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;
    let peer_id: Vec<u8> = decode_base58(&peer_id_str)?;
    let node_id = OpaquePeerId(peer_id);
    let current_nonce = rpc
        .system_account_next_index(&signer.public_key().into())
        .await?;
    let current_header = rpc.chain_get_header(None).await?.unwrap();
    if verbose {
        println!("Curent nonce: {:?}", current_nonce);
        println!("Curent block header: {:?}", current_header);
    }
    let ext_params = Params::new()
        .mortal(&current_header, 8)
        .nonce(current_nonce)
        .build();
    let tx = vrs_metadata::codegen::tx()
        .nucleus()
        .upload_nucleus_wasm(nucleus_id, node_id, digest);
    let signed_tx = api.tx().create_signed(&tx, &signer, ext_params).await?;
    let tx_bytes = signed_tx.into_encoded();
    if verbose {
        println!("[+] Signed raw transaction: {}", hex::encode(&tx_bytes));
    }
    use subxt::backend::legacy::rpc_methods::Bytes;
    let params = rpc_params![Bytes(tx_bytes), Bytes(file_content.to_vec())];
    let deploy_result: String = rpc_client.request("nucleus_deploy", params).await?;
    println!("Transaction submitted: {:?}", deploy_result);
    Ok(())
}
