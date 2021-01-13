use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use std::{convert::TryFrom, string::String};
use substrate_api_client::{
    compose_call, compose_extrinsic_offline, extrinsic::xt_primitives::UncheckedExtrinsicV4,
    node_metadata::Metadata, Api, XtStatus,
};

fn main() {
    // instantiate an Api that connects to the given address
    let url = "127.0.0.1:9944";
    // if no signer is set in the whole program, we need to give to Api a specific type instead of an associated type
    // as during compilation the type needs to be defined.
    let signer = AccountKeyring::Bob.pair();

    // sets up api client and retrieves the node metadata
    let api = Api::new(format!("ws://{}", url)).set_signer(signer.clone());
    // gets the current nonce of Bob so we can increment it manually later
    let mut nonce = api.get_nonce().unwrap();

    // data from the node required in extrinsic
    let meta = Metadata::try_from(api.get_metadata()).unwrap();
    let genesis_hash = api.genesis_hash;
    let spec_version = api.runtime_version.spec_version;
    let transaction_version = api.runtime_version.transaction_version;

    // Example bytes to add
    let bytes_to_add: Vec<u8> = vec![1, 2, 3, 4];
    // Example CID for the example bytes added vec![1, 2, 3, 4]
    let cid = String::from("QmRgctVSR8hvGRDLv7c5H7BCji7m1VXRfEE1vW78CFagD7").into_bytes();

    // Example multiaddr to connect IPFS with
    let multiaddr = String::from(
        "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
    )
    .into_bytes();

    // Example Peer Id
    let peer_id = String::from("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ").into_bytes();

    // Create input for all calls
    let calls = vec![
        ("create_twin", peer_id.clone()),
    ];

    // Create Extinsics and listen for all calls
    for call in calls {
        println!("\n Creating Extrinsic for {}", call.0);
        let _call = compose_call!(meta, "TemplateModule", call.0, call.1);
        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
            signer,
            _call,
            nonce,
            Era::Immortal,
            genesis_hash,
            genesis_hash,
            spec_version,
            transaction_version
        );

        let blockh = api
            .send_extrinsic(xt.hex_encode(), XtStatus::Finalized)
            .unwrap();
        println!("Transaction got finalized in block {:?}", blockh);
        nonce += 1;
    }
}
