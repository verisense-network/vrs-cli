use clap::Parser;
use serde::{Deserialize, Serialize};
use subxt::backend::rpc::RpcClient;
use subxt::config::substrate::{AccountId32, H256};
use subxt::rpc_params;

// TODO invole `vrs_primitives` after stable release
#[derive(Serialize, Deserialize, Debug)]
pub struct NucleusInfo {
    pub name: Vec<u8>,
    pub manager: AccountId32,
    pub wasm_hash: H256,
    pub wasm_version: u32,
    pub wasm_location: Option<sp_core::OpaquePeerId>,
    pub current_event: u64,
    pub root_state: H256,
    pub validators: Vec<AccountId32>,
}

#[derive(Debug, Clone, Parser)]
#[command(name = "info", about = "Query information of a nucleus")]
pub struct NucleusInfoCmd {
    #[arg(
        long = "id",
        value_name = "ID",
        help = "The nucleus ID to install WASM"
    )]
    nucleus_id: String,
}

impl NucleusInfoCmd {
    /// Run the command
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let rpc = options.get_rpc();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async { info(rpc, self.nucleus_id.clone()).await })
    }
}

async fn info(rpc: impl AsRef<str>, nucleus_id: String) -> anyhow::Result<()> {
    let rpc_client = RpcClient::from_url(rpc).await?;
    let params = rpc_params![nucleus_id];
    let nucleus: NucleusInfo = rpc_client.request("nucleus_info", params.clone()).await?;
    println!("Name: {}", String::from_utf8_lossy(&nucleus.name));
    println!("Owner: {}", crate::account::to_ss58check(nucleus.manager.0));
    println!("WASM version: {}", nucleus.wasm_version);
    println!("WASM checksum: {}", nucleus.wasm_hash);
    println!("Validators: ");
    nucleus
        .validators
        .into_iter()
        .for_each(|v| println!("  - {}", crate::account::to_ss58check(v.0)));
    Ok(())
}
