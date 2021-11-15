//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_finality_grandpa_rpc::{GrandpaApi, GrandpaRpcHandler};
use sc_consensus_babe_rpc::BabeRpcHandler;
use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;
use tfchain_runtime::{opaque::Block, AccountId, Balance, BlockNumber, Hash, Index};
use sc_consensus_babe::{Config, Epoch};
use sp_keystore::SyncCryptoStorePtr;
use sc_consensus_epochs::SharedEpochChanges;

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
	pub select_chain: SC,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
    /// The GRANDPA specific dependencies
    pub grandpa: GrandpaDeps<B>,
}

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// BABE protocol config.
	pub babe_config: Config,
	/// BABE pending epoch changes.
	pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
	/// The keystore that manages the keys of the node.
	pub keystore: SyncCryptoStorePtr,
}

/// Dependencies for Grandpa rpc endpoints
pub struct GrandpaDeps<B> {
    /// voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// GRANDPA justification stream.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor for GRANDPA subscription manager
    pub executor: SubscriptionTaskExecutor,
    /// Proof provider for GRANDPA
    pub finality_proof_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, SC, B>(deps: FullDeps<C, P, SC, B>) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: sp_consensus_babe::BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
    SC: sp_consensus::SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        select_chain,
        deny_unsafe,
        babe,
        grandpa,
    } = deps;
    let BabeDeps {
		keystore,
		babe_config,
		shared_epoch_changes,
	} = babe;
    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        executor,
        finality_proof_provider,
    } = grandpa;

    io.extend_with(
		SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe))
	);

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));

    io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(BabeRpcHandler::new(
		client.clone(),
		shared_epoch_changes,
		keystore,
		babe_config,
		select_chain,
		deny_unsafe,
	)));

    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `io.extend_with(YourRpcTrait::to_delegate(YourRpcStruct::new(ReferenceToClient, ...)));`
    io.extend_with(GrandpaApi::to_delegate(GrandpaRpcHandler::new(
        shared_authority_set,
        shared_voter_state,
        justification_stream,
        executor,
        finality_proof_provider,
    )));

    io
}
