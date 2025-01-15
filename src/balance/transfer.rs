use clap::Parser;
use subxt::utils::{AccountId32, MultiAddress};
use subxt::{OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::Keypair;

#[derive(Debug, Clone, Parser)]
#[command(about = "Transfer some amount of $VRS to another account")]
pub struct TransferCmd {
    #[arg(long, help = "The recipient account address")]
    to: String,

    #[arg(
        long,
        help = "Amount of $VRS to transfer, the number is fixed-point 128-bit. E.g. 1 $VRS = 1_000_000_000_000_000_000"
    )]
    amount: u128,
}

impl TransferCmd {
    /// Run the command
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let rpc = options.get_rpc();
        let key_file = options.get_keyfile();
        let signer = crate::account::read_key(key_file)?;
        let to = crate::account::to_account(&self.to)?;
        runtime.block_on(async { transfer(signer, rpc, to, self.amount).await })
    }
}

async fn transfer(
    signer: Keypair,
    rpc: impl AsRef<str>,
    to: AccountId32,
    amount: u128,
) -> anyhow::Result<()> {
    let api = OnlineClient::<SubstrateConfig>::from_url(rpc).await?;
    let balance_transfer_tx = crate::runtime::codegen::tx()
        .balances()
        .transfer_allow_death(MultiAddress::Id(to), amount);
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&balance_transfer_tx, &signer)
        .await?
        .wait_for_finalized_success()
        .await?;
    let transfer_event = events.find_first::<crate::runtime::balances::events::Transfer>()?;
    if let Some(event) = transfer_event {
        println!(
            "Transfered {} to {}",
            event.amount,
            crate::account::to_ss58check(event.to.0),
        );
    }
    Ok(())
}
