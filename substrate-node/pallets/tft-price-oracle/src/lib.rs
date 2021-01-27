#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, debug,
	traits::{
		Get,
	},
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
	},
};
use system::{ensure_none};

use sp_std::prelude::*;

use lite_json::json::JsonValue;
use sp_runtime::{
    offchain::{http, Duration},
};

use fixed::{types::U16F16};

use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"tft!");

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

pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	// Add other types and constants required to configure this pallet.
    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    type Call: From<Call<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TFTPriceModule {
		// Token price
		TftPrice: U16F16;
	}
}

decl_event! {
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		PriceStored(U16F16, AccountId),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		OffchainSignedTxError,
		NoLocalAcctForSigning
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn set_prices(origin, price: U16F16){
			let _ = ensure_signed(origin)?;
			TftPrice::put(price);
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn submit_price(origin, price: U16F16) {
			ensure_none(origin)?;
			TftPrice::put(price);
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			match Self::offchain_signed_tx() {
				Ok(_) => debug::info!("worker executed"),
				Err(err) => debug::info!("err: {:?}", err)
			}
        }
	}
}

impl<T: Trait> Module<T> {
    /// Fetch current price and return the result in cents.
    fn fetch_price() -> Result<f64, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
 
        let request =
            http::Request::get("https://min-api.cryptocompare.com/data/price?fsym=TFT&tsyms=USD");

        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;

        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;

        // Let's check the status code before we proceed to reading the response.
        if response.code != 200 {
            debug::warn!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        let body = response.body().collect::<Vec<u8>>();

        // Create a str slice from the body.
        let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
            debug::warn!("No UTF8 body");
            http::Error::Unknown
        })?;

        let price = match Self::parse_price(body_str) {
            Some(price) => Ok(price),
            None => {
                debug::warn!("Unable to extract price from the response: {:?}", body_str);
                Err(http::Error::Unknown)
            }
        }?;

        debug::info!("TFT price: {:?}", price);

        Ok(price)
	}
	
	fn parse_price(price_str: &str) -> Option<f64> {
        let val = lite_json::parse_json(price_str);
        let price = val.ok().and_then(|v| match v {
            JsonValue::Object(obj) => {
                let mut chars = "USD".chars();
                obj.into_iter()
                    .find(|(k, _)| k.iter().all(|k| Some(*k) == chars.next()))
                    .and_then(|v| match v.1 {
                        JsonValue::Number(number) => Some(number),
                        _ => None,
                    })
            }
            _ => None,
        })?;

		let exp = price.fraction_length.checked_sub(2).unwrap_or(0);
		
		let price = 
			(price.integer * 10_i64.pow(exp as u32) + price.fraction as i64) as f64
			/ (100 * 10_u32.pow(exp as u32)) as f64;
        Some(price)
	}
	
	fn offchain_signed_tx() -> Result<(), Error<T>> {
		let price = match Self::fetch_price() {
			Ok(v) => v,
			Err(_) => return Err(<Error<T>>::OffchainSignedTxError)
		};

		let price_to_fixed = U16F16::from_num(price);

        let signer = Signer::<T, T::AuthorityId>::any_account();

        let result = signer.send_signed_transaction(|_acct| {
            Call::set_prices(price_to_fixed)
        });
    
        // Display error if the signed tx fails.
        // Display error if the signed tx fails.
        if let Some((acc, res)) = result {
            if res.is_err() {
                debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
                return Err(<Error<T>>::OffchainSignedTxError);
            }
            // Transaction is sent successfully
            return Ok(());
        }
        // The case of `None`: no account is available for sending
        debug::error!("No local account available");
        return Err(<Error<T>>::OffchainSignedTxError)
    }
}
