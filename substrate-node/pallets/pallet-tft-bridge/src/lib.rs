#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

mod tft_bridge;
mod types;
pub mod weights;
pub mod migrations;

// Definition of the pallet logic, to be aggregated at runtime definition
// through `construct_runtime`.
#[frame_support::pallet]
pub mod pallet {
    use super::{
        types::{BurnTransaction, MintTransaction, RefundTransaction, StellarSignature},
        weights::WeightInfo,
    };
    use super::*;
    use frame_support::{
        pallet_prelude::{*, OptionQuery},
        traits::{Currency, EnsureOrigin, OnUnbalanced, ReservableCurrency},
    };
    use frame_system::{self as system, ensure_signed, pallet_prelude::*};
    use sp_runtime::SaturatedConversion;
    use sp_std::prelude::*;

    // balance type using reservable currency type
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;
    pub type NegativeImbalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::NegativeImbalance;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn validator_accounts)]
    pub type Validators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn fee_account)]
    pub type FeeAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn mint_transactions)]
    pub type MintTransactions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>,
        MintTransaction<T::AccountId, T::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn executed_mint_transactions)]
    pub type ExecutedMintTransactions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>,
        MintTransaction<T::AccountId, T::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn burn_transactions)]
    pub type BurnTransactions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BurnTransaction<T::AccountId, T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn executed_burn_transactions)]
    pub type ExecutedBurnTransactions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BurnTransaction<T::AccountId, T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn refund_transactions)]
    pub type RefundTransactions<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, RefundTransaction<T::BlockNumber>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn executed_refund_transactions)]
    pub type ExecutedRefundTransactions<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, RefundTransaction<T::BlockNumber>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn burn_transaction_id)]
    pub type BurnTransactionID<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn withdraw_fee)]
    pub type WithdrawFee<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn deposit_fee)]
    pub type DepositFee<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pallet_version)]
    pub type PalletVersion<T> = StorageValue<_, types::StorageVersion, ValueQuery>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency type for this pallet.
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Handler for the unbalanced decrement when slashing (burning collateral)
        type Burn: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Origin for restricted extrinsics
        /// Can be the root or another origin configured in the runtime
        type RestrictedOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // Retry interval for expired transactions
        type RetryInterval: Get<u32>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // Minting events
        MintTransactionProposed(Vec<u8>, T::AccountId, u64),
        MintTransactionVoted(Vec<u8>),
        MintCompleted(MintTransaction<T::AccountId, T::BlockNumber>),
        MintTransactionExpired(Vec<u8>, u64, T::AccountId),
        // Burn events
        BurnTransactionCreated(u64, T::AccountId, Vec<u8>, u64),
        BurnTransactionProposed(u64, Vec<u8>, u64),
        BurnTransactionSignatureAdded(u64, StellarSignature),
        BurnTransactionReady(u64),
        BurnTransactionProcessed(BurnTransaction<T::AccountId, T::BlockNumber>),
        BurnTransactionExpired(u64, Option<T::AccountId>, Vec<u8>, u64),
        // Refund events
        RefundTransactionCreated(Vec<u8>, Vec<u8>, u64),
        RefundTransactionsignatureAdded(Vec<u8>, StellarSignature),
        RefundTransactionReady(Vec<u8>),
        RefundTransactionProcessed(RefundTransaction<T::BlockNumber>),
        RefundTransactionExpired(Vec<u8>, Vec<u8>, u64),
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorExists,
        ValidatorNotExists,
        TransactionValidatorExists,
        TransactionValidatorNotExists,
        MintTransactionExists,
        MintTransactionAlreadyExecuted,
        MintTransactionNotExists,
        BurnTransactionExists,
        BurnTransactionNotExists,
        BurnSignatureExists,
        EnoughBurnSignaturesPresent,
        RefundSignatureExists,
        BurnTransactionAlreadyExecuted,
        RefundTransactionNotExists,
        RefundTransactionAlreadyExecuted,
        EnoughRefundSignaturesPresent,
        NotEnoughBalanceToSwap,
        AmountIsLessThanWithdrawFee,
        AmountIsLessThanDepositFee,
        WrongParametersProvided,
        InvalidStellarPublicKey,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub validator_accounts: Option<Vec<T::AccountId>>,
        pub fee_account: Option<T::AccountId>,
        pub withdraw_fee: u64,
        pub deposit_fee: u64,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                validator_accounts: None,
                fee_account: None,
                withdraw_fee: Default::default(),
                deposit_fee: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            if let Some(validator_accounts) = &self.validator_accounts {
                Validators::<T>::put(validator_accounts);
            }

            if let Some(ref fee_account) = self.fee_account {
                FeeAccount::<T>::put(fee_account);
            }
            WithdrawFee::<T>::put(self.withdraw_fee);
            DepositFee::<T>::put(self.deposit_fee)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(block: T::BlockNumber) {
            let current_block_u64: u64 = block.saturated_into::<u64>();

            for (tx_id, mut tx) in BurnTransactions::<T>::iter() {
                let tx_block_u64: u64 = tx.block.saturated_into::<u64>();
                // if x blocks have passed since the tx got submitted
                // we can safely assume this tx is fault
                // add the faulty tx to the expired tx list
                if current_block_u64 - tx_block_u64 >= T::RetryInterval::get().into() {
                    // reset signatures and sequence number
                    tx.signatures = Vec::new();
                    tx.sequence_number = 0;
                    tx.block = block;

                    // update tx in storage
                    BurnTransactions::<T>::insert(&tx_id, &tx);

                    // Emit event
                    Self::deposit_event(Event::BurnTransactionExpired(tx_id, tx.source, tx.target, tx.amount));
                }
            }

            for (tx_id, mut tx) in RefundTransactions::<T>::iter() {
                let tx_block_u64: u64 = tx.block.saturated_into::<u64>();
                // if x blocks have passed since the tx got submitted
                // we can safely assume this tx is fault
                // add the faulty tx to the expired tx list
                if current_block_u64 - tx_block_u64 >= T::RetryInterval::get().into() {
                    // reset signatures and sequence number
                    tx.signatures = Vec::new();
                    tx.sequence_number = 0;
                    tx.block = block;

                    // update tx in storage
                    RefundTransactions::<T>::insert(&tx_id, &tx);

                    // Emit event
                    Self::deposit_event(Event::RefundTransactionExpired(
                        tx_id, tx.target, tx.amount,
                    ));
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::add_bridge_validator())]
        pub fn add_bridge_validator(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::add_validator_account(target)
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_bridge_validator())]
        pub fn remove_bridge_validator(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::remove_validator_account(target)
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_fee_account())]
        pub fn set_fee_account(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            FeeAccount::<T>::set(Some(target));
            Ok(().into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::set_withdraw_fee())]
        pub fn set_withdraw_fee(origin: OriginFor<T>, amount: u64) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            WithdrawFee::<T>::set(amount);
            Ok(().into())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_deposit_fee())]
        pub fn set_deposit_fee(origin: OriginFor<T>, amount: u64) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            DepositFee::<T>::set(amount);
            Ok(().into())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::swap_to_stellar())]
        pub fn swap_to_stellar(
            origin: OriginFor<T>,
            target_stellar_address: Vec<u8>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let source = ensure_signed(origin)?;
            Self::burn_tft(source, target_stellar_address, amount)
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_or_vote_mint_transaction())]
        pub fn propose_or_vote_mint_transaction(
            origin: OriginFor<T>,
            transaction: Vec<u8>,
            target: T::AccountId,
            amount: u64,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::propose_or_vote_stellar_mint_transaction(validator, transaction, target, amount)
        }

        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_burn_transaction_or_add_sig())]
        pub fn propose_burn_transaction_or_add_sig(
            origin: OriginFor<T>,
            transaction_id: u64,
            target: Vec<u8>,
            amount: u64,
            signature: Vec<u8>,
            stellar_pub_key: Vec<u8>,
            sequence_number: u64,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::propose_stellar_burn_transaction_or_add_sig(
                validator,
                transaction_id,
                target,
                amount,
                signature,
                stellar_pub_key,
                sequence_number,
            )
        }

        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::set_burn_transaction_executed())]
        pub fn set_burn_transaction_executed(
            origin: OriginFor<T>,
            transaction_id: u64,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::set_stellar_burn_transaction_executed(validator, transaction_id)
        }

        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::create_refund_transaction_or_add_sig())]
        pub fn create_refund_transaction_or_add_sig(
            origin: OriginFor<T>,
            tx_hash: Vec<u8>,
            target: Vec<u8>,
            amount: u64,
            signature: Vec<u8>,
            stellar_pub_key: Vec<u8>,
            sequence_number: u64,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::create_stellar_refund_transaction_or_add_sig(
                validator,
                tx_hash,
                target,
                amount,
                signature,
                stellar_pub_key,
                sequence_number,
            )
        }

        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::set_refund_transaction_executed())]
        pub fn set_refund_transaction_executed(
            origin: OriginFor<T>,
            tx_hash: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::set_stellar_refund_transaction_executed(validator, tx_hash)
        }
    }
}
