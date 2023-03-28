// use codec::Decode;
// use futures::StreamExt;
// use sp_keyring::AccountKeyring;
// use std::time::Duration;
// use subxt::{tx::PairSigner, OnlineClient, SubstrateConfig};

// #[subxt::subxt(runtime_metadata_path = "artifacts/devnet.scale")]
// pub mod devnet {}

// pub mod client;

// /// Subscribe to all events, and then manually look through them and
// /// pluck out the events that we care about.
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt::init();

//     // Create a client to use:
//     let api = OnlineClient::<SubstrateConfig>::new().await?;

//     // Subscribe to any events that occur:
//     let mut event_sub = api.events().subscribe().await?;

//     // While this subscription is active, balance transfers are made somewhere:
//     tokio::task::spawn({
//         let api = api.clone();
//         async move {
//             let signer = PairSigner::new(AccountKeyring::Alice.pair());
//             let mut transfer_amount = 1_000_000_000;

//             // Make small balance transfers from Alice to Bob in a loop:
//             loop {
//                 let transfer_tx = devnet::tx()
//                     .balances()
//                     .transfer(AccountKeyring::Bob.to_account_id().into(), transfer_amount);
//                 api.tx()
//                     .sign_and_submit_default(&transfer_tx, &signer)
//                     .await
//                     .unwrap();

//                 tokio::time::sleep(Duration::from_secs(10)).await;
//                 transfer_amount += 100_000_000;
//             }
//         }
//     });

//     // Our subscription will see the events emitted as a result of this:
//     while let Some(events) = event_sub.next().await {
//         let events = events?;
//         let block_hash = events.block_hash();

//         // We can dynamically decode events:
//         println!("  Dynamic event details: {block_hash:?}:");
//         for event in events.iter() {
//             let event = event?;
//             let is_balance_transfer = event
//                 .as_event::<devnet::balances::events::Transfer>()?
//                 .is_some();
//             let pallet = event.pallet_name();
//             let variant = event.variant_name();
//             println!("    {pallet}::{variant} (is balance transfer? {is_balance_transfer})");
//         }

//         // Or we can find the first transfer event, ignoring any others:
//         let transfer_event = events.find_first::<devnet::balances::events::Transfer>()?;

//         if let Some(ev) = transfer_event {
//             println!("  - Balance transfer success: value: {:?}", ev.amount);
//         } else {
//             println!("  - No balance transfer event found in this block");
//         }
//     }

//     Ok(())
// }

pub mod client;
pub mod runtimes;

use crate::runtimes::types::BlockNumber;
use crate::runtimes::types::Hash;
use client::{KeyPair, KeyType, Runtime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let phrase = "oyster orient plunge devote light wrap hold mother essence casino rebel distance";
    // let p = KeyPair::from_phrase(KeyType::Sr25519, phrase, None)
    //     .expect("failed to get key from phrase");

    // let seed = "0x9917ea107aca8e9c29f4530413b41333ada03cf39fede45cde611b943e2e8dd1";
    // let _ = KeyPair::from_phrase(KeyType::Sr25519, seed, None);

    // let cl = client::Client::new(
    //     String::from("wss://tfchain.dev.grid.tf:443"),
    //     Runtime::Devnet,
    // )
    // .await?;

    let cl = client::Client::new(
        String::from("wss://tfchain.test.grid.tf:443"),
        Runtime::Testnet,
    )
    .await?;

    let twin = cl.get_twin_by_id(105, None).await?;
    println!("got twin: {:?}", twin);

    // let twin = cl.get_twin_by_id(1, None).await?;
    // println!("got twin: {:?}", twin);

    let farm = cl.get_farm_by_id(1, None).await?;
    println!("got farm: {:?}", farm);

    // let block_1 = cl
    //     .get_block_hash(Some(BlockNumber::from(5808046 as u64)))
    //     .await?;
    // let cl2 =
    //     client::Client::new(String::from("wss://tfchain.grid.tf:443"), Runtime::Testnet).await?;
    // let node2 = cl2.get_node_by_id(1, block_1).await?;
    // println!("got node: {:?}", node2);

    // println!("block 1 hash {:?}", block_1);
    // let node = cl.get_node_by_id(1, block_1).await?;
    // println!("got node: {:?}", node);

    // let account = "5HmARi4eGLhb9hvFrbCC5F8dCNRTS8MWKc6xbmPUS1cnKD7c"
    //     .parse::<AccountId32>()
    //     .unwrap();

    // let balance_at_block_1 = cl.get_balance(&account, block_1).await;
    // println!("balance at block 1: {:?}", balance_at_block_1);

    // let balance = cl.get_balance(&account, None).await?;
    // println!("balance at current block: {:?}", balance);

    // let _ = cl.get_contract_by_id(915).await?;

    // let node = cl.get_node_by_id(15).await?;
    // println!("got node: {:?}", node);

    Ok(())
}
