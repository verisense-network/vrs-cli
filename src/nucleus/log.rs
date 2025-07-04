use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(about = "Query logs of a nucleus")]
pub struct CheckNucleusLogCmd {
    #[arg(long = "id", value_name = "ID", help = "The nucleus ID")]
    nucleus_id: String,
}

impl CheckNucleusLogCmd {
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let rpc = options.get_rpc();
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async { query_logs(rpc, self.nucleus_id.clone()).await })
    }
}

async fn query_logs(rpc_url: impl AsRef<str>, nucleus_id: String) -> anyhow::Result<()> {
    let rpc_url = rpc_url
        .as_ref()
        .replace("ws://", "http://")
        .replace("wss://", "https://");
    let url = format!("{}/{}/logs", rpc_url, nucleus_id);
    let client = reqwest::Client::new();
    let logs = client.get(&url).send().await?.text().await?;
    println!("{}", logs);
    Ok(())
}
