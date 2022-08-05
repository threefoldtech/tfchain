#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    weights::Pays,
};
use frame_system::{
    offchain::{SendSignedTransaction, Signer},
};
use lite_json::json::JsonValue;
use log;
use sp_runtime::offchain::{http, Duration};
use sp_runtime::traits::SaturatedConversion;
use sp_std::{boxed::Box, vec::Vec};
mod ringbuffer;

use ringbuffer::{RingBufferTrait, RingBufferTransient};
use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"tft!");

// type BufferIndex = u16;

#[cfg(test)]
mod tests;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        ensure,
        traits::{EnsureOrigin,},
    };
    use frame_system::{
        ensure_signed,
        offchain::{AppCrypto, CreateSignedTransaction,},
    };

    use crate::KEY_TYPE;
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
    {
        // type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        // Add other types and constants required to configure this pallet.
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        type Call: From<Call<Self>>;
        /// Origin for restricted extrinsics
        /// Can be the root or another origin configured in the runtime
        type RestrictedOrigin: EnsureOrigin<Self::Origin>;
    }

	#[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    // pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
    pub enum Event<T: Config> {
        PriceStored(u32),
        OffchainWorkerExecuted(T::AccountId),
        AveragePriceStored(u32),
    }

	#[pallet::error]
    // pub enum Error for Module<T: Config> {
    pub enum Error<T> {
        ErrFetchingPrice,
        OffchainSignedTxError,
        NoLocalAcctForSigning,
        AccountUnauthorizedToSetPrice,
    }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn tft_price)]
    pub type TftPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn last_block_set)]
    pub type LastBlockSet<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

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
    #[pallet::getter(fn allowed_origin)]
    pub type AllowedOrigin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    //TODO
    // add_extra_genesis {
    //     config(allowed_origin): Option<T::AccountId>;

    //     build(|_config| {
    //         AllowedOrigin::<T>::set(_config.allowed_origin.clone());
    //     });
    // }

	#[pallet::call]   // <-- Step 6. code block will replace this.
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn set_prices(
            origin: OriginFor<T>, 
            price: u32,
            block_number: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;
            if let Some(allowed_origin) = AllowedOrigin::<T>::get() {
                ensure!(allowed_origin == address, Error::<T>::AccountUnauthorizedToSetPrice);
                Self::calculate_and_set_price(price, block_number)?;
            }
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn set_allowed_origin(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            AllowedOrigin::<T>::set(Some(target));
            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            match Self::offchain_signed_tx(block_number) {
                Ok(_) => log::info!("offchain worker done."),
                Err(err) => log::info!("err: {:?}", err)
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    fn calculate_and_set_price(
        price: u32,
        block_number: T::BlockNumber,
    ) -> DispatchResultWithPostInfo {
        log::info!("price {:?}", price);

        LastBlockSet::<T>::put(block_number);
        TftPrice::<T>::put(price);
        Self::deposit_event(Event::PriceStored(price));

        log::info!("storing average now");
        let mut queue = Self::queue_transient();
        queue.push(price);
        let average = Self::calc_avg();

        log::info!("average price {:?}", average);
        AverageTftPrice::<T>::put(average);
        Self::deposit_event(Event::AveragePriceStored(average));

        Ok(Pays::No.into())
    }

    /// Fetch current price and return the result in cents.
    fn fetch_price() -> Result<u32, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

        let request =
            http::Request::get("https://min-api.cryptocompare.com/data/price?fsym=3ft&tsyms=USD");

        let pending = request.deadline(deadline).send().map_err(|_| {
            log::error!("IO error");
            http::Error::IoError
        })?;

        let response = pending.try_wait(deadline).map_err(|_| {
            log::error!("Deadline reached");
            http::Error::DeadlineReached
        })??;

        // Let's check the status code before we proceed to reading the response.
        if response.code != 200 {
            log::error!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        // Next we want to fully read the response body and collect it to a vector of bytes.
        // Note that the return object allows you to read the body in chunks as well
        // with a way to control the deadline.
        let body = response.body().collect::<Vec<u8>>();

        // Create a str slice from the body.
        let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
            log::warn!("No UTF8 body");
            http::Error::Unknown
        })?;

        let price = match Self::parse_price(body_str) {
            Some(price) => Ok(price),
            None => {
                log::warn!("Unable to extract price from the response: {:?}", body_str);
                Err(http::Error::Unknown)
            }
        }?;

        log::warn!("Got price: {} cents", price);

        Ok(price)
    }

    fn offchain_signed_tx(block_number: T::BlockNumber) -> Result<(), Error<T>> {
        let last_block_set: T::BlockNumber = LastBlockSet::<T>::get();
        // Fetch the price every 1 minutes
        if block_number.saturated_into::<u64>() - last_block_set.saturated_into::<u64>() < 10 {
            return Ok(());
        }
        let price = match Self::fetch_price() {
            Ok(v) => v,
            Err(err) => {
                log::error!("err while fetching price: {:?}", err);
                return Err(<Error<T>>::ErrFetchingPrice);
            }
        };

        let signer = Signer::<T, T::AuthorityId>::any_account();

        let result = signer.send_signed_transaction(|_acct| Call::set_prices {
            price,
            block_number,
        });

        // Display error if the signed tx fails.
        if let Some((acc, res)) = result {
            if res.is_err() {
                log::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
                return Err(<Error<T>>::OffchainSignedTxError);
            }
            // Transaction is sent successfully
            return Ok(());
        }
        // The case of `None`: no account is available for sending
        log::error!("No local account available");
        return Err(<Error<T>>::OffchainSignedTxError);
    }

    /// Parse the price from the given JSON string using `lite-json`.
    ///
    /// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
    pub fn parse_price(price_str: &str) -> Option<u32> {
        let val = lite_json::parse_json(price_str);
        let price = match val.ok()? {
            JsonValue::Object(obj) => {
                let (_, v) = obj
                    .into_iter()
                    .find(|(k, _)| k.iter().copied().eq("USD".chars()))?;
                match v {
                    JsonValue::Number(number) => number,
                    _ => return None,
                }
            }
            _ => return None,
        };

        let exp = price.fraction_length.saturating_sub(3);
        Some(price.integer as u32 * 1000 + (price.fraction / 10_u64.pow(exp)) as u32)
    }

    fn queue_transient() -> Box<dyn RingBufferTrait<u32>> {
        Box::new(RingBufferTransient::<
            u32,
            <Self as Store>::BufferRange,
            <Self as Store>::TftPriceHistory,
        >::new())
    }

    fn calc_avg() -> u32 {
        let queue = Self::queue_transient();
        let items = queue.get_all_values();
        items.iter().fold(0_u32, |a, b| a.saturating_add(*b)) / items.len() as u32
    }
}
