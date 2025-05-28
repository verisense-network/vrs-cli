use clap::Parser;
use subxt::backend::rpc::RpcClient;
use subxt::{OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::Keypair;

#[derive(Debug, Clone, Parser)]
#[command(about = "Create a new nucleus instance on Verisense network")]
pub struct CreateNucleusCmd {
    name: String,

    #[arg(
        short = 'c',
        long,
        help = "The number of subnet members to run this nucleus"
    )]
    capacity: u8,

    #[arg(long, help = "Indicates the nucleus is for running an AI agent.")]
    agent: bool,
}

impl CreateNucleusCmd {
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let rpc = options.get_rpc();
        let key_file = options.get_keyfile();
        let signer = crate::account::read_key(key_file)?;
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            submit_tx(rpc, signer, self.name.clone(), self.capacity, self.agent).await
        })
    }
}

async fn submit_tx(
    rpc_url: impl AsRef<str>,
    signer: Keypair,
    nucleus_name: String,
    capacity: u8,
    agent: bool,
) -> anyhow::Result<()> {
    let rpc_client = RpcClient::from_url(rpc_url.as_ref()).await?;
    let api = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;
    let tx = crate::runtime::tx().nucleus().create_nucleus(
        nucleus_name.as_bytes().to_vec(),
        None,
        capacity,
        agent,
    );
    let result = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, &signer)
        .await?;
    let events = result.wait_for_finalized_success().await?;
    for ev in events.iter().flatten() {
        if let Some(ev) = ev.as_event::<crate::runtime::nucleus::events::NucleusCreated>()? {
            println!("Nucleus created.");
            println!("  ID: {}", crate::account::to_ss58check(ev.id.0));
            println!("  Name: {}", std::str::from_utf8(&ev.name).unwrap());
            println!("  Capacity: {}", ev.capacity);
        }
    }
    Ok(())
}
