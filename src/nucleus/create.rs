use clap::Parser;
use subxt::backend::rpc::RpcClient;
use subxt::config::substrate::H256;
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
}

impl CreateNucleusCmd {
    pub fn run(&self, options: crate::cli::Options) -> anyhow::Result<()> {
        let rpc = options.get_rpc();
        let key_file = options.get_keyfile();
        let signer = crate::key::read_key(key_file)?;
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async { submit_tx(rpc, signer, self.name.clone(), self.capacity).await })
    }
}

async fn submit_tx(
    rpc_url: impl AsRef<str>,
    signer: Keypair,
    nucleus_name: String,
    capacity: u8,
) -> anyhow::Result<()> {
    let rpc_client = RpcClient::from_url(rpc_url.as_ref()).await?;
    let api = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;
    let tx = vrs_metadata::codegen::tx().nucleus().create_nucleus(
        nucleus_name.as_bytes().to_vec(),
        // TODO, the pallet will remove this
        H256::zero(),
        None,
        capacity,
    );
    let result = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, &signer)
        .await?;
    let events = result.wait_for_finalized_success().await?;
    for ev in events.iter().flatten() {
        if let Some(ev) = ev.as_event::<vrs_metadata::codegen::nucleus::events::NucleusCreated>()? {
            println!("Nucleus created.");
            println!("  ID: {}", ev.id);
            println!("  Name: {}", std::str::from_utf8(&ev.name).unwrap());
            println!("  Capacity: {}", ev.capacity);
        }
    }
    Ok(())
}

// use subxt_core::config::DefaultExtrinsicParamsBuilder as Params;
// use subxt_core::metadata;
// use subxt_core::tx;
// use subxt_signer::sr25519::dev;

// async fn send_to_substrate(
//     nucleus_name: String,
//     capacity: u8,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // Gather some other information about the chain that we'll need to construct valid extrinsics:
//     let state = tx::ClientState::<SubstrateConfig> {
//         metadata: {
//             let metadata_bytes = include_bytes!("../../verisense/metadata/metadata.scale");
//             metadata::decode_from(&metadata_bytes[..]).unwrap()
//         },
//         genesis_hash: {
//             let h = "1ca3dd31d5d886127f1476537ba88065ce82cafb6e8ae9c190022e1ec3fb0c56";
//             let bytes = hex::decode(h).unwrap();
//             H256::from_slice(&bytes)
//         },
//         runtime_version: tx::RuntimeVersion {
//             spec_version: 101,
//             transaction_version: 1,
//         },
//     };
//     let call = substrate::tx()
//         .nucleus() // Replace with your actual pallet name
//         .create_nucleus(
//             nucleus_name.as_bytes().to_vec(),
//             H256::zero(),
//             None,
//             capacity,
//         );

//     tx::validate(&call, &state.metadata).unwrap();

//     // We can build a signed transaction:
//     let signed_call = tx::create_signed(&call, &state, &dev::bob(), Default::default()).unwrap();

//     // And log it:
//     println!("Tx: 0x{}", hex::encode(signed_call.encoded()));
//     let rpc_client = RpcClient::from_url(RPC_HOST).await?;
//     // Use this to construct our RPC methods:
//     // let rpc = LegacyRpcMethods::<SubstrateConfig>::new(rpc_client.clone());
//     // Create a new client
//     let api = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;
//     let xt = subxt::tx::SubmittableExtrinsic::from_bytes(api, signed_call.encoded().to_vec());
//     let result = xt.submit_and_watch().await?;
//     let events = result.wait_for_finalized_success().await?;
//     // println!("Transaction finalized: events {:?}", events);
//     for ev in events.iter().flatten() {
//         if let Some(ev) = ev.as_event::<substrate::nucleus::events::NucleusCreated>()? {
//             println!("Nucleus created.");
//             println!("  id: {}", ev.id);
//             println!("  name: {}", std::str::from_utf8(&ev.name).unwrap());
//             println!("  capacity: {}", ev.capacity);
//         }
//     }

//     use std::str::FromStr;
//     let call = substrate::tx().nucleus().register(
//         subxt::utils::AccountId32::from_str("5EDDNaRtENUDSyovzucxZoQHCea2H7CLJxFCFytcpGXgt9ke")
//             .unwrap(),
//         substrate::runtime_types::sp_core::sr25519::vrf::VrfSignature {
//             pre_output: [0u8; 32],
//             proof: [0u8; 64],
//         },
//     );
//     let signed_call = tx::create_signed(&call, &state, &dev::alice(), Default::default()).unwrap();
//     let unsigned_tx = tx::create_partial_signed(&call, &state, Default::default())
//         .expect("couldn't create paritial transaction");
//     let raw = unsigned_tx.signer_payload();
//     let signature = sign_tx(raw);
//     let addr = dev::alice().public_key().to_address();
//     let signature = subxt::utils::MultiSignature::Sr25519(signature);
//     let signed = unsigned_tx.sign_with_address_and_signature(&addr, &signature);

//     println!("Register Tx: 0x{}", hex::encode(signed.encoded()));
//     // let api = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client.clone()).await?;
//     // let xt = subxt::tx::SubmittableExtrinsic::from_bytes(api, signed_call.encoded().to_vec());
//     // let result = xt.submit_and_watch().await?;
//     // let events = result.wait_for_finalized_success().await?;
//     // println!("Transaction finalized: events {:?}", events);
//     // for ev in events.iter().flatten() {
//     //     println!("{:?}", ev);
//     // }

//     Ok(())
// }

// fn sign_tx(raw: Vec<u8>) -> [u8; 64] {
//     dev::alice().sign(&raw).0
//     // let signature = sign_payload
//     //     .using_encoded(|x| keystore.sr25519_sign(key_type, &public, &x))?
//     //     .ok_or("fail to sign `register` tx signature, please check the keystore")?;
//     // Ok(signature.0)
// }
