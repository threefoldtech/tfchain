#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    traits::{Get, EnsureOrigin}, 
    weights::{Pays},
    dispatch::DispatchResultWithPostInfo,
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
};
use log;

use sp_std::prelude::*;

use codec::{Decode, Encode};
use sp_runtime::traits::SaturatedConversion;
use sp_runtime::offchain::{http, Duration};

use substrate_fixed::types::U16F16;
mod ringbuffer;

use ringbuffer::{RingBufferTrait, RingBufferTransient};
use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"tft!");
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValueStruct {
    value: U16F16,
}
type BufferIndex = u16;
#[cfg(test)]
mod tests;
pub mod crypto {
    use crate::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };

    app_crypto!(sr25519, KEY_TYPE);

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
}

// #[cfg(test)]
// mod tests;

pub trait Config: system::Config + CreateSignedTransaction<Call<Self>> {
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    // Add other types and constants required to configure this pallet.
    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    type Call: From<Call<Self>>;
    /// Origin for restricted extrinsics
    /// Can be the root or another origin configured in the runtime
    type RestrictedOrigin: EnsureOrigin<Self::Origin>;}

decl_storage! {
    trait Store for Module<T: Config> as TFTPriceModule {
        // Token price
        pub TftPrice: U16F16;
        LastBlockSet: T::BlockNumber;
        pub AverageTftPrice: U16F16;
        pub TftPriceHistory get(fn get_value): map hasher(twox_64_concat) BufferIndex => ValueStruct;
        BufferRange get(fn range): (BufferIndex, BufferIndex) = (0, 0);
        pub AllowedOrigin get(fn allowed_origin): T::AccountId;
    }

    add_extra_genesis {
        config(allowed_origin): T::AccountId;

        build(|_config| {
            AllowedOrigin::<T>::set(_config.allowed_origin.clone());
        });
    }
}

decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
        PriceStored(U16F16),
        OffchainWorkerExecuted(AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        ErrFetchingPrice,
        OffchainSignedTxError,
        NoLocalAcctForSigning,
        AccountUnauthorizedToSetPrice,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 100_000_000 + T::DbWeight::get().writes(3)]
        pub fn set_prices(origin, price: U16F16, block_number: T::BlockNumber) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;
            ensure!(AllowedOrigin::<T>::get() == address, Error::<T>::AccountUnauthorizedToSetPrice);
            Self::calculate_and_set_price(price, block_number)
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(3)]
        pub fn set_allowed_origin(origin, target: T::AccountId) {
            T::RestrictedOrigin::ensure_origin(origin)?;
            AllowedOrigin::<T>::set(target);
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            match Self::offchain_signed_tx(block_number) {
                Ok(_) => log::info!("offchain worker done."),
                Err(err) => log::info!("err: {:?}", err)
            }
        }
    }
}

use serde::Deserialize;

#[derive(Deserialize, Default)]
struct PriceInfo {
    #[serde(rename = "USD")]
    price: f64,
}

impl<T: Config> Module<T> {
    fn calculate_and_set_price(price: U16F16, block_number: T::BlockNumber) -> DispatchResultWithPostInfo {
        log::info!("price {:?}", price);

        LastBlockSet::<T>::put(block_number);
        TftPrice::put(price);
        Self::deposit_event(RawEvent::PriceStored(price));

        log::info!("storing average now");
        let mut queue = Self::queue_transient();
        queue.push(ValueStruct { value: price });
        let average = Self::calc_avg();

        log::info!("average price {:?}", average);
        AverageTftPrice::put(average);

        Ok(Pays::No.into())
    }

    /// Fetch current price and return the result in cents.
    fn fetch_price() -> Result<f64, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

        let request = http::Request::get("https://min-api.cryptocompare.com/data/price?fsym=3ft&tsyms=USD");

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

        let body = response.body().collect::<Vec<u8>>();

        // Create a str slice from the body.
        let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
            log::error!("No UTF8 body");
            http::Error::Unknown
        })?;

        let price_info: PriceInfo = serde_json::from_str(&body_str).map_err(|_| {
            log::error!("Error while decoding");
            http::Error::Unknown
        })?;

        Ok(price_info.price)
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

        let price_to_fixed = U16F16::from_num(price);

        let signer = Signer::<T, T::AuthorityId>::any_account();

        let result =
            signer.send_signed_transaction(|_acct| Call::set_prices(price_to_fixed, block_number));

        // Display error if the signed tx fails.
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

    fn queue_transient() -> Box<dyn RingBufferTrait<ValueStruct>> {
        Box::new(RingBufferTransient::<
            ValueStruct,
            <Self as Store>::BufferRange,
            <Self as Store>::TftPriceHistory,
        >::new())
    }

    fn calc_avg() -> U16F16 {
        let mut sum: U16F16 = U16F16::from_num(0);
        let mut counter = U16F16::from_num(0);

        let queue = Self::queue_transient();
        let items = queue.get_all_values();
        for item in items {
            let ValueStruct { value } = item;
            if value >= 0 {
                sum += value;
                counter += U16F16::from_num(1);
            }
        }
        if counter == U16F16::from_num(0) {
            return U16F16::from_num(0);
        }
        sum / counter
    }
}
