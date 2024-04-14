#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
mod ringbuffer;
mod tft_price;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use crate::tft_price::KEY_TYPE;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*, traits::EnsureOrigin,
    };
    use frame_system::{
        ensure_signed,
        offchain::{AppCrypto, CreateSignedTransaction},
        pallet_prelude::*,
    };
    use log;
    use scale_info::prelude::format;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    use sp_std::convert::TryFrom;

    app_crypto!(sr25519, KEY_TYPE);

    type BufferIndex = u16;
    pub struct AuthId;

    // implemented for ocw-runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for AuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + CreateSignedTransaction<Call<Self>>
        + pallet_authorship::Config
        + pallet_session::Config
    {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        // Add other types and constants required to configure this pallet.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        type Call: From<Call<Self>>;
        /// Origin for restricted extrinsics
        /// Can be the root or another origin configured in the runtime
        type RestrictedOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PriceStored(u32),
        OffchainWorkerExecuted(T::AccountId),
        AveragePriceStored(u32),
        AveragePriceIsAboveMaxPrice(u32, u32),
        AveragePriceIsBelowMinPrice(u32, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        ErrFetchingPrice,
        OffchainSignedTxError,
        NoLocalAcctForSigning,
        AccountUnauthorizedToSetPrice,
        MaxPriceBelowMinPriceError,
        MinPriceAboveMaxPriceError,
        IsNotAnAuthority,
        WrongAuthority,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn tft_price)]
    pub type TftPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn last_block_set)]
    pub type LastBlockSet<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn average_tft_price)]
    pub type AverageTftPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_value)]
    pub type TftPriceHistory<T> = StorageMap<_, Blake2_128Concat, BufferIndex, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn range)]
    pub type BufferRange<T> = StorageValue<_, (BufferIndex, BufferIndex), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn min_tft_price)]
    pub type MinTftPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn max_tft_price)]
    pub type MaxTftPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::set_prices())]
        pub fn set_prices(
            origin: OriginFor<T>,
            price: u32,
            block_number: BlockNumberFor<T>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            ensure!(
                Self::is_validator(address),
                Error::<T>::AccountUnauthorizedToSetPrice
            );

            Self::calculate_and_set_price(price, block_number)
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_min_tft_price())]
        pub fn set_min_tft_price(origin: OriginFor<T>, price: u32) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            ensure!(
                price < MaxTftPrice::<T>::get(),
                Error::<T>::MinPriceAboveMaxPriceError
            );
            MinTftPrice::<T>::put(price);
            Ok(().into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::set_max_tft_price())]
        pub fn set_max_tft_price(origin: OriginFor<T>, price: u32) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            ensure!(
                price > MinTftPrice::<T>::get(),
                Error::<T>::MaxPriceBelowMinPriceError
            );
            MaxTftPrice::<T>::put(price);
            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            match Self::offchain_signed_tx(block_number) {
                Ok(_) => log::info!("offchain worker done."),
                Err(err) => log::error!("{:?}", err),
            }
        }
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub min_tft_price: u32,
        pub max_tft_price: u32,
        pub _data: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            MinTftPrice::<T>::put(self.min_tft_price);
            MaxTftPrice::<T>::put(self.max_tft_price);
        }
    }
}
