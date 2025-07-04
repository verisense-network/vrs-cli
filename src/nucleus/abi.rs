use clap::Parser;
use subxt::backend::rpc::RpcClient;
use subxt::rpc_params;

#[derive(Debug, Clone, Parser)]
#[command(name = "abi", about = "Fetch ABI of a nucleus")]
pub struct NucleusAbiCmd {
    #[arg(
        long = "id",
        value_name = "ID",
        help = "The nucleus ID to install WASM"
    )]
    nucleus_id: String,

    #[arg(long = "pretty", help = "Print ABI in pretty format")]
    pretty: bool,
}

impl NucleusAbiCmd {
    /// Run the command
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let rpc = options.get_rpc();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async { abi(rpc, self.nucleus_id.clone(), self.pretty).await })
    }
}

async fn abi(rpc: impl AsRef<str>, nucleus_id: String, pretty: bool) -> anyhow::Result<()> {
    let rpc_client = RpcClient::from_url(rpc).await?;
    let params = rpc_params![nucleus_id];
    let abi: serde_json::Value = rpc_client.request("nucleus_abi", params).await?;
    if pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&abi).unwrap_or_else(|_| "N/A".to_string())
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&abi).unwrap_or_else(|_| "N/A".to_string())
        );
    }
    Ok(())
}
