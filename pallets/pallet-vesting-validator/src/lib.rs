#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure,
	traits::{Vec},
};
use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::{DispatchResult};
use codec::{Decode, Encode};
use sp_runtime::traits::SaturatedConversion;

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Config>::AccountId,
	{
		TransactionProposed(Vec<u8>, AccountId),
		TransactionSignatureAdded(Vec<u8>, Vec<u8>, AccountId),
		TransactionReady(Vec<u8>),
		TransactionRemoved(Vec<u8>),
		TransactionExpired(Vec<u8>),
		TransactionFailed(Vec<u8>, Vec<u8>),
	}
);

decl_error! {
	/// Error for the vesting module.
	pub enum Error for Module<T: Config> {
		ValidatorExists,
		ValidatorNotExists,
		TransactionValidatorExists,
		TransactionValidatorNotExists,
		TransactionExists,
		SimilarTransactionExists,
		TransactionNotExists,
		SignatureExists
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct StellarTransaction <BlockNumber> {
	pub block: BlockNumber,
	pub signatures: Vec<Vec<u8>>
}

decl_storage! {
	trait Store for Module<T: Config> as VestingValidatorModule {
		pub Validators get(fn validator_accounts): Vec<T::AccountId>;

		pub Transactions get(fn transactions): map hasher(blake2_128_concat) Vec<u8> => StellarTransaction<T::BlockNumber>;
		pub TransactionsByEscrow get(fn transactions_by_escrow): map hasher(blake2_128_concat) T::AccountId => Vec<u8>;
		
		pub ExpiredTransactions get(fn expired_transactions): map hasher(blake2_128_concat) Vec<u8> => StellarTransaction<T::BlockNumber>;
		pub ExecutedTransactions get(fn executed_transactions): map hasher(blake2_128_concat) Vec<u8> => StellarTransaction<T::BlockNumber>;
		pub FailedTransactions get(fn failed_transactions): map hasher(blake2_128_concat) Vec<u8> => StellarTransaction<T::BlockNumber>;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
		
		#[weight = 10_000]
		fn add_validator(origin, target: T::AccountId){
            ensure_root(origin)?;
            Self::add_validator_account(target)?;
		}
		
		#[weight = 10_000]
		fn remove_validator(origin, target: T::AccountId){
            ensure_root(origin)?;
            Self::remove_validator_account(target)?;
		}
		
		#[weight = 10_000]
		fn propose_transaction(origin, target: T::AccountId, transaction: Vec<u8>){
            let _ = ensure_signed(origin)?;
            Self::propose_stellar_transaction(target, transaction)?;
		}

		#[weight = 10_000]
		fn add_sig_transaction(origin, transaction: Vec<u8>, signature: Vec<u8>){
            let validator = ensure_signed(origin)?;
            Self::add_sig_stellar_transaction(validator, transaction, signature)?;
		}

		#[weight = 10_000]
		fn remove_transaction(origin, transaction: Vec<u8>){
            let validator = ensure_signed(origin)?;
            Self::set_stellar_transaction_executed(validator, transaction)?;
		}

		#[weight = 10_000]
		fn report_failed_transaction(origin, transaction: Vec<u8>, reason: Vec<u8>){
            let validator = ensure_signed(origin)?;
            Self::set_stellar_transaction_failed(validator, transaction, reason)?;
		}

		fn on_finalize(block: T::BlockNumber) {
			let current_block_u64: u64 = block.saturated_into::<u64>();

			for (tx_id, tx) in Transactions::<T>::iter() {
				let tx_block_u64: u64 = tx.block.saturated_into::<u64>();
				// if 1000 blocks have passed since the tx got submitted
				// we can safely assume this tx is fault
				// add the faulty tx to the expired tx list
				if current_block_u64 - tx_block_u64 >= 1000 {
					// Remove tx from storage
					Transactions::<T>::remove(tx_id.clone());
					// search for the transaction in the transactions by escrow map and delete it there as well
					for (key, tx) in TransactionsByEscrow::<T>::iter() {
						if tx == tx_id {
							TransactionsByEscrow::<T>::remove(key);
						}
					}
					// Insert into expired transactions list
					ExpiredTransactions::<T>::insert(tx_id.clone(), tx);
					// Emit an expired event so validators can choose to retry
					Self::deposit_event(RawEvent::TransactionExpired(tx_id));
				}
			}
		}
	}
}

impl<T: Config> Module<T> {
	pub fn add_validator_account(target: T::AccountId) -> DispatchResult {
		let mut validators = Validators::<T>::get();

		match validators.binary_search(&target) {
			Ok(_) => Err(Error::<T>::ValidatorExists.into()),
			// If the search fails, the caller is not a member and we learned the index where
			// they should be inserted
			Err(index) => {
				validators.insert(index, target.clone());
				Validators::<T>::put(validators);
				Ok(())
			}
		}
	}

	pub fn remove_validator_account(target: T::AccountId) -> DispatchResult {
		let mut validators = Validators::<T>::get();

		match validators.binary_search(&target) {
			Ok(index) => {
				validators.remove(index);
				Validators::<T>::put(validators);
				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}

	pub fn propose_stellar_transaction(target: T::AccountId, tx: Vec<u8>) -> DispatchResult {
		// make sure we don't duplicate the transaction
		ensure!(!Transactions::<T>::contains_key(tx.clone()), Error::<T>::TransactionExists);
		
		// make sure there can only be one transaction for each escrow at a time
		ensure!(!TransactionsByEscrow::<T>::contains_key(target.clone()), Error::<T>::SimilarTransactionExists);
		
		let now = <frame_system::Module<T>>::block_number();
		let stellar_tx = StellarTransaction {
			block: now,
			signatures: Vec::new()
		};

		Transactions::<T>::insert(tx.clone(), &stellar_tx);
		TransactionsByEscrow::<T>::insert(&target, &tx);

		Self::deposit_event(RawEvent::TransactionProposed(tx, target));

		Ok(())
	}

	// This will remove the transaction and add it to the executed transactions list
	pub fn set_stellar_transaction_executed(origin: T::AccountId, tx_id: Vec<u8>) -> DispatchResult {
		// make sure we don't duplicate the transaction
		ensure!(Transactions::<T>::contains_key(tx_id.clone()), Error::<T>::TransactionNotExists);

		
		let validators = Validators::<T>::get();
		match validators.binary_search(&origin) {
			Ok(_) => {
				let tx = Transactions::<T>::get(tx_id.clone());

				// Store it as an executed transaction
				ExecutedTransactions::<T>::insert(tx_id.clone(), &tx);

				// search for the transaction in the transactions by escrow map and delete it there as well
				for (key, tx) in TransactionsByEscrow::<T>::iter() {
					if tx == tx_id {
						TransactionsByEscrow::<T>::remove(key);
					}
				}

				// Remove it from the current transactions list
				Transactions::<T>::remove(tx_id.clone());

				Self::deposit_event(RawEvent::TransactionRemoved(tx_id));

				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}

	// This will remove the transaction and add it to the failed transactions list
	pub fn set_stellar_transaction_failed(origin: T::AccountId, tx_id: Vec<u8>, reason: Vec<u8>) -> DispatchResult {
		// make sure we don't duplicate the transaction
		ensure!(Transactions::<T>::contains_key(tx_id.clone()), Error::<T>::TransactionNotExists);

		let validators = Validators::<T>::get();
		match validators.binary_search(&origin) {
			Ok(_) => {
				let tx = Transactions::<T>::get(tx_id.clone());

				// Store it as a failed transaction
				FailedTransactions::<T>::insert(tx_id.clone(), &tx);

				// search for the transaction in the transactions by escrow map and delete it there as well
				for (key, tx) in TransactionsByEscrow::<T>::iter() {
					if tx == tx_id {
						TransactionsByEscrow::<T>::remove(key);
					}
				}

				// Remove it from the current transactions list
				Transactions::<T>::remove(tx_id.clone());

				Self::deposit_event(RawEvent::TransactionFailed(tx_id, reason));

				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}

	pub fn add_sig_stellar_transaction(origin: T::AccountId, tx_id: Vec<u8>, signature: Vec<u8>) -> DispatchResult {
		// make sure tx exists
		ensure!(Transactions::<T>::contains_key(tx_id.clone()), Error::<T>::TransactionExists);
		
		let validators = Validators::<T>::get();
		match validators.binary_search(&origin) {
			Ok(_) => {				
				let mut tx = Transactions::<T>::get(&tx_id.clone());
				// check if the signature already exists
				ensure!(!tx.signatures.iter().any(|c| c == &signature), Error::<T>::SignatureExists);

				// if more then then the half of all validators
				// submitted their signature we can emit an event that a transaction
				// is ready to be submitted to the stellar network
				if tx.signatures.len() > validators.len() / 2 {
					Self::deposit_event(RawEvent::TransactionReady(tx_id));
				} else {
					// add the signature
					tx.signatures.push(signature.clone());
					Transactions::<T>::insert(tx_id.clone(), &tx);
					Self::deposit_event(RawEvent::TransactionSignatureAdded(tx_id, signature, origin));
				}

				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}
}
