#[subxt::subxt(
    runtime_metadata_path = "artifacts/devnet.scale",
    substitute_type(
        type = "frame_support::storage::bounded_vec::BoundedVec",
        with = "::sp_std::vec::Vec"
    )
)]
pub mod devnet {}

use super::types;
pub use devnet::runtime_types::frame_system::AccountInfo;
pub use devnet::runtime_types::pallet_balances::AccountData;
pub use devnet::runtime_types::pallet_smart_contract::types::Contract;
pub use devnet::runtime_types::pallet_tfgrid::{
    farm::FarmName,
    interface::{InterfaceIp, InterfaceMac, InterfaceName},
    node::{Location, SerialNumber},
    types::Twin as TwinData,
};
use devnet::runtime_types::sp_core::bounded::bounded_vec::BoundedVec;
pub use devnet::runtime_types::tfchain_support::types::{
    Farm as FarmData, Interface, Node as NodeData, PublicConfig, PublicIP as PublicIpData,
};
use subxt::ext::sp_core::H256;
use subxt::utils::AccountId32;
use subxt::Error;

pub type Twin = TwinData<AccountId32>;

pub type Farm = FarmData<FarmName>;

pub type InterfaceOf = Interface<InterfaceName, InterfaceMac, BoundedVec<InterfaceIp>>;
pub type Node = NodeData<Location, InterfaceOf, SerialNumber>;

use crate::client::{Client, KeyPair};

pub use devnet::tft_bridge_module::events::BurnTransactionReady;
pub use devnet::tft_bridge_module::events::BurnTransactionSignatureAdded;
pub use devnet::tft_bridge_module::events::MintTransactionProposed;

pub type SystemAccountInfo = AccountInfo<u32, AccountData<u128>>;

pub async fn create_twin(
    cl: &Client,
    kp: &KeyPair,
    relay: Option<String>,
    pk: Option<String>,
) -> Result<u32, Error> {
    let create_twin_tx = devnet::tx().tfgrid_module().create_twin(
        relay.map(|r| BoundedVec(r.as_bytes().to_vec())),
        pk.map(|r| BoundedVec(r.as_bytes().to_vec())),
    );

    let signer = kp.signer();

    let create_twin = cl
        .api
        .tx()
        .sign_and_submit_then_watch_default(&create_twin_tx, &signer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let twin_create_event =
        create_twin.find_first::<devnet::tfgrid_module::events::TwinStored>()?;

    if let Some(event) = twin_create_event {
        Ok(event.0.id)
    } else {
        Err(Error::Other(String::from("failed to create twin")))
    }
}

pub async fn update_twin(
    cl: &Client,
    kp: &KeyPair,
    relay: Option<String>,
    pk: Option<&[u8]>,
) -> Result<H256, Error> {
    let update_twin_tx = devnet::tx().tfgrid_module().update_twin(
        relay.map(|r| BoundedVec(r.as_bytes().to_vec())),
        pk.map(|r| BoundedVec(r.to_vec())),
    );

    let signer = kp.signer();

    let update_twin = cl
        .api
        .tx()
        .sign_and_submit_then_watch_default(&update_twin_tx, &signer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let twin_create_event =
        update_twin.find_first::<devnet::tfgrid_module::events::TwinUpdated>()?;

    if let Some(_) = twin_create_event {
        Ok(update_twin.block_hash())
    } else {
        Err(Error::Other(String::from("failed to create twin")))
    }
}

pub async fn sign_terms_and_conditions(
    cl: &Client,
    kp: &KeyPair,
    document_link: String,
    document_hash: String,
) -> Result<H256, Error> {
    let sign_tandc_tx = devnet::tx().tfgrid_module().user_accept_tc(
        BoundedVec(document_link.as_bytes().to_vec()),
        BoundedVec(document_hash.as_bytes().to_vec()),
    );

    let signer = kp.signer();

    let sign_tandc = cl
        .api
        .tx()
        .sign_and_submit_then_watch_default(&sign_tandc_tx, &signer)
        .await?
        .wait_for_finalized_success()
        .await?;

    Ok(sign_tandc.block_hash())
}

pub async fn get_twin_by_id(cl: &Client, id: u32) -> Result<Option<types::Twin>, Error> {
    Ok(cl
        .api
        .storage()
        .await?
        .fetch(&devnet::storage().tfgrid_module().twins(id))
        .await?
        .map(types::Twin::from))
}

pub async fn get_twin_id_by_account(
    cl: &Client,
    account: AccountId32,
) -> Result<Option<u32>, Error> {
    cl.api
        .storage()
        .await?
        .fetch(
            &devnet::storage()
                .tfgrid_module()
                .twin_id_by_account_id(account),
        )
        .await
}

pub async fn get_contract_by_id(cl: &Client, id: u64) -> Result<Option<types::Contract>, Error> {
    Ok(cl
        .api
        .storage()
        .await?
        .fetch(&devnet::storage().smart_contract_module().contracts(id))
        .await?
        .map(types::Contract::from))
}

pub async fn get_node_by_id(cl: &Client, id: u32) -> Result<Option<types::TfgridNode>, Error> {
    Ok(cl
        .api
        .storage()
        .await?
        .fetch(&devnet::storage().tfgrid_module().nodes(id))
        .await?
        .map(types::TfgridNode::from))
}

pub async fn get_farm_by_id(cl: &Client, id: u32) -> Result<Option<types::TfgridFarm>, Error> {
    Ok(cl
        .api
        .storage()
        .await?
        .fetch(&devnet::storage().tfgrid_module().farms(id))
        .await?
        .map(types::TfgridFarm::from))
}

pub async fn get_block_hash(
    cl: &Client,
    block_number: Option<types::BlockNumber>,
) -> Result<Option<types::Hash>, Error> {
    cl.api.rpc().block_hash(block_number).await
}

pub async fn get_balance(
    cl: &Client,
    account: &AccountId32,
) -> Result<Option<types::SystemAccountInfo>, Error> {
    Ok(cl
        .api
        .storage()
        .await?
        .fetch(&devnet::storage().system().account(account))
        .await?
        .map(types::SystemAccountInfo::from))
}
