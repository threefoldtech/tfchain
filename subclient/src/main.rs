use codec::Encode;
use sp_keyring::AccountKeyring;
use substrate_subxt::{Call, ClientBuilder, EventsDecoder, NodeTemplateRuntime, PairSigner};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Signer for the extrinsic
    let signer = PairSigner::<NodeTemplateRuntime, _>::new(AccountKeyring::Alice.pair());
    // API client, default to connect to 127.0.0.1:9944
    let client = ClientBuilder::<NodeTemplateRuntime>::new().build().await?;

    // Example CID for the example bytes added vec![1, 2, 3, 4]
    let cid = String::from("QmRgctVSR8hvGRDLv7c5H7BCji7m1VXRfEE1vW78CFagD7").into_bytes();
    // Example multiaddr to connect IPFS with
    let multiaddr = String::from(
        "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
    )
    .into_bytes();
    // Example Peer Id
    let peer_id = String::from("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ").into_bytes();

    // Begin to submit extrinsics
    // ipfs_add_bytes
    let add_bytes = client
        .watch(
            AddBytesCall {
                peer_id: vec![1, 2, 3, 5],
            },
            &signer,
        )
        .await?;
    println!("\nResult for ipfs_add_bytes: {:?}", add_bytes);

    // // ipfs_cat_bytes
    // let cat_bytes = client
    //     .watch(CatBytesCall { cid: cid.clone() }, &signer)
    //     .await?;

    // data: Vec<u8>,
    Ok(())
}

impl Call<NodeTemplateRuntime> for AddBytesCall {
    const MODULE: &'static str = "TemplateModule";
    const FUNCTION: &'static str = "create_twin";
    fn events_decoder(_decoder: &mut EventsDecoder<NodeTemplateRuntime>) {}
}

#[derive(Encode)]
pub struct CatBytesCall {
    cid: Vec<u8>,
}

#[derive(Encode)]
pub struct AddBytesCall {
    peer_id: Vec<u8>,
}

// // builds with 1.48
// use anyhow::{Context, Result};
// use sp_keyring::AccountKeyring;
// use substrate_subxt::{extrinsic, ClientBuilder, NodeTemplateRuntime, PairSigner, RuntimeVersion};

// //use substrate_api_client;

// #[tokio::main]
// async fn main() -> Result<()> {
//     // //println!("Hello, world!");

//     // //let signer = PairSigner::new(AccountKeyring::Alice.pair());

//     let client = ClientBuilder::<NodeTemplateRuntime>::new()
//         .set_url("ws://127.0.0.1:9944")
//         .build()
//         .await
//         .context("failed to build client")?;

//     //client.create_twin();
//     //extrinsic::create_signed(c, genesis_hash: T::Hash, nonce: T::Index, call: Encoded, signer: &(dyn Signer<T> + Send + Sync))
//     let metadata = client.metadata();
//     println!("{}", metadata.pretty());
//     let module = metadata.module("TemplateModule")?;

//     println!("{:?}", module);

//     Ok(())
// }
