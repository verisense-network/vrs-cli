pub const RPC_HOST: &str = "ws://127.0.0.1:9944";
// const RPC_HOST: &str = "wss://alpha-devnet.verisense.network";

// Generate an interface that we can use from the node's metadata.
// #[subxt::subxt(runtime_metadata_path = "metadata.scale")]
#[subxt::subxt(runtime_metadata_insecure_url = "ws://127.0.0.1:9944")]
// #[subxt::subxt(runtime_metadata_insecure_url = "wss://alpha-devnet.verisense.network")]
pub mod substrate {}
