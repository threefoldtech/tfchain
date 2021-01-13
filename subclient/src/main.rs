// builds with 1.48
use anyhow::{Context, Result};
use sp_keyring::AccountKeyring;
use substrate_subxt::{extrinsic, ClientBuilder, NodeTemplateRuntime, PairSigner, RuntimeVersion};

use node_template_runtime;

#[tokio::main]
async fn main() -> Result<()> {
    //println!("Hello, world!");

    //let signer = PairSigner::new(AccountKeyring::Alice.pair());

    let client = ClientBuilder::<NodeTemplateRuntime>::new()
        .set_url("ws://127.0.0.1:9944")
        .build()
        .await
        .context("failed to build client")?;

    //client.create_twin();
    //extrinsic::create_signed(c, genesis_hash: T::Hash, nonce: T::Index, call: Encoded, signer: &(dyn Signer<T> + Send + Sync))
    let genesis = client.metadata();
    println!("{}", genesis.pretty());

    Ok(())
}
