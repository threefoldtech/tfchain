use super::*;
use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::Pays};
use frame_system::{
    offchain::{SendSignedTransaction, SignMessage, Signer},
    pallet_prelude::BlockNumberFor,
};
use ringbuffer::{RingBufferTrait, RingBufferTransient};
use scale_info::prelude::format;
use serde_json::Value;
use sp_core::{crypto::KeyTypeId, offchain::Duration};
use sp_runtime::{
    offchain::http,
    traits::{Convert, SaturatedConversion},
};
use sp_std::{boxed::Box, vec::Vec};
use substrate_fixed::types::U32F32;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"aura");

pub const SRC_CODE: &str = "USDC";
pub const SRC_ISSUER: &str = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";
pub const DST_TYPE: &str = "credit_alphanum4";
pub const DST_ISSUER: &str = "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47";
pub const DST_CODE: &str = "TFT";
pub const DST_AMOUNT: u32 = 100;

impl<T: Config> Pallet<T> {
    pub(crate) fn calculate_and_set_price(
        price: u32,
        block_number: BlockNumberFor<T>,
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
    pub(crate) fn fetch_price() -> Result<u32, http::Error> {
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

    pub(crate) fn offchain_signed_tx(block_number: BlockNumberFor<T>) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::any_account();

        // Only allow the author of the next block to trigger price fetching
        match Self::is_next_block_author(&signer) {
            Ok(_) => (),
            Err(_) => return Ok(()),
        }

        let last_block_set: BlockNumberFor<T> = LastBlockSet::<T>::get();
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

    pub(crate) fn queue_transient() -> Box<dyn RingBufferTrait<u32>> {
        Box::new(RingBufferTransient::<u32, BufferRange<T>, TftPriceHistory<T>>::new())
    }

    pub(crate) fn calc_avg() -> u32 {
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
    pub(crate) fn is_next_block_author(
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

    pub(crate) fn is_validator(account: T::AccountId) -> bool {
        let validators = <pallet_session::Pallet<T>>::validators();

        validators.iter().any(|validator| {
            match <T as pallet_session::Config>::ValidatorIdOf::convert(account.clone()) {
                Some(signer) => &signer == validator,
                None => false,
            }
        })
    }
}
