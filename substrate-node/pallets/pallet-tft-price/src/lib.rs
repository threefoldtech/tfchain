#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{dispatch::DispatchResultWithPostInfo, weights::Pays};
use frame_system::offchain::{SendSignedTransaction, SignMessage, Signer};
use log;
use sp_runtime::offchain::{http, Duration};
use sp_runtime::traits::{Convert, SaturatedConversion};
use sp_std::{boxed::Box, vec::Vec};
mod ringbuffer;
use ringbuffer::{RingBufferTrait, RingBufferTransient};
use scale_info::prelude::format;
use serde_json::Value;
use sp_core::crypto::KeyTypeId;
use substrate_fixed::types::U32F32;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"aura");

const SRC_CODE: &str = "USDC";
const SRC_ISSUER: &str = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";
const DST_TYPE: &str = "credit_alphanum4";
const DST_ISSUER: &str = "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47";
const DST_CODE: &str = "TFT";
const DST_AMOUNT: u32 = 100;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, traits::EnsureOrigin};
    use frame_system::{
        ensure_signed,
        offchain::{AppCrypto, CreateSignedTransaction},
    };

    use crate::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    use sp_std::convert::TryFrom;
    use sp_std::marker::PhantomData;

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
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
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
    #[pallet::getter(fn min_tft_price)]
    pub type MinTftPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn max_tft_price)]
    pub type MaxTftPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn set_prices(
            origin: OriginFor<T>,
            price: u32,
            block_number: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            ensure!(
                Self::is_validator(address),
                Error::<T>::AccountUnauthorizedToSetPrice
            );

            Self::calculate_and_set_price(price, block_number)
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn set_min_tft_price(origin: OriginFor<T>, price: u32) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            ensure!(
                price < MaxTftPrice::<T>::get(),
                Error::<T>::MinPriceAboveMaxPriceError
            );
            MinTftPrice::<T>::put(price);
            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
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
        fn offchain_worker(block_number: T::BlockNumber) {
            match Self::offchain_signed_tx(block_number) {
                Ok(_) => log::info!("offchain worker done."),
                Err(err) => log::error!("{:?}", err),
            }
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub min_tft_price: u32,
        pub max_tft_price: u32,
        pub _data: PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                min_tft_price: 10,
                max_tft_price: 1000,
                _data: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            MinTftPrice::<T>::put(self.min_tft_price);
            MaxTftPrice::<T>::put(self.max_tft_price);
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

        let min = Self::min_tft_price();
        if average < min {
            log::info!("average price {:?} is below min price {:?} !", average, min);
            Self::deposit_event(Event::AveragePriceIsBelowMinPrice(average, min));
        }

        let max = Self::max_tft_price();
        if average > max {
            log::info!("average price {:?} is above max price {:?} !", average, max);
            Self::deposit_event(Event::AveragePriceIsAboveMaxPrice(average, max));
        }

        Ok(Pays::No.into())
    }

    /// Fetch current price and return the result in mUSD.
    fn fetch_price() -> Result<u32, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

        let request_url = format!(
            "https://horizon.stellar.org/paths/strict-receive?source_assets={}%3A{}&destination_asset_type={}&destination_asset_issuer={}&destination_asset_code={}&destination_amount={}",
            SRC_CODE, SRC_ISSUER, DST_TYPE, DST_ISSUER, DST_CODE, DST_AMOUNT,
        );

        let request = http::Request::get(request_url.as_str());

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

        let price = match Self::parse_lowest_price_from_request(body_str) {
            Some(price) => Ok(price),
            None => {
                log::warn!("Unable to extract price from the response: {:?}", body_str);
                Err(http::Error::Unknown)
            }
        }?;

        // Get price for 1 TFT in mUSD
        let tft_usd = (U32F32::from_num(price) / U32F32::from_num(DST_AMOUNT))
            .round()
            .to_num::<u32>();
        log::info!("Got price: {} mUSD", tft_usd);

        Ok(tft_usd)
    }

    fn offchain_signed_tx(block_number: T::BlockNumber) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::any_account();

        // Only allow the author of the next block to trigger the billing
        Self::is_next_block_author(&signer)?;

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

    /// Parse the lowest price from the given JSON string using `serde_json`.
    ///
    /// Returns `None` when parsing failed or `Some(price in mUSD)` when parsing is successful.
    pub fn parse_lowest_price_from_request(price_str: &str) -> Option<u32> {
        let data: Value = serde_json::from_str(price_str).ok()?;
        let records_array = data.get("_embedded")?.get("records")?;

        let prices: Vec<U32F32> = records_array
            .as_array()?
            .into_iter()
            .map(|item| {
                let val = item.get("source_amount")?;
                let str = val.as_str()?;
                let p = str.parse::<U32F32>().ok()?;
                Some(p)
            })
            .map(|x| {
                if let Some(p) = x {
                    p
                } else {
                    U32F32::from_num(f32::NAN)
                }
            })
            .collect();

        let lowest = prices.into_iter().reduce(U32F32::min)?;
        // convert to mUSD
        Some(((lowest) * 1000).round().to_num::<u32>())
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
        let sum = items.iter().fold(0_u32, |a, b| a.saturating_add(*b));
        (U32F32::from_num(sum) / U32F32::from_num(items.len()))
            .round()
            .to_num::<u32>()
    }

    // Validates if the given signer is the next block author based on the validators in session
    // This can be used if an extrinsic should be refunded by the author in the same block
    // It also requires that the keytype inserted for the offchain workers is the validator key
    fn is_next_block_author(
        signer: &Signer<T, <T as Config>::AuthorityId>,
    ) -> Result<(), Error<T>> {
        let author = <pallet_authorship::Pallet<T>>::author();
        let validators = <pallet_session::Pallet<T>>::validators();

        // Sign some arbitrary data in order to get the AccountId, maybe there is another way to do this?
        let signed_message = signer.sign_message(&[0]);
        if let Some(signed_message_data) = signed_message {
            if let Some(block_author) = author {
                let validator =
                    <T as pallet_session::Config>::ValidatorIdOf::convert(block_author.clone())
                        .ok_or(Error::<T>::IsNotAnAuthority)?;

                let validator_count = validators.len();
                let author_index = (validators.iter().position(|a| a == &validator).unwrap_or(0)
                    + 1)
                    % validator_count;

                let signer_validator_account =
                    <T as pallet_session::Config>::ValidatorIdOf::convert(
                        signed_message_data.0.id.clone(),
                    )
                    .ok_or(Error::<T>::IsNotAnAuthority)?;

                if signer_validator_account != validators[author_index] {
                    return Err(Error::<T>::WrongAuthority);
                }
            }
        }

        Ok(().into())
    }

    fn is_validator(account: T::AccountId) -> bool {
        let validators = <pallet_session::Pallet<T>>::validators();

        validators.iter().any(|validator| {
            match <T as pallet_session::Config>::ValidatorIdOf::convert(account.clone()) {
                Some(signer) => &signer == validator,
                None => false,
            }
        })
    }
}
