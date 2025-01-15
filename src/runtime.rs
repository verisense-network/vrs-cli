pub use codegen::*;

#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod codegen {}
