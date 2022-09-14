use crate::{Config, Error};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

pub const MIN_DOCUMENT_LINK_LENGTH: u32 = 1;
pub const MAX_DOCUMENT_LINK_LENGTH: u32 = 50;

pub const MIN_DOCUMENT_HASH_LENGTH: u32 = 1;
pub const MAX_DOCUMENT_HASH_LENGTH: u32 = 50;

/// Terms and conditions.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct TermsAndConditions<T: Config> {
    pub account_id: T::AccountId,
    pub timestamp: u64,
    pub document_link: BoundedVec<u8, ConstU32<MAX_DOCUMENT_LINK_LENGTH>>,
    pub document_hash: BoundedVec<u8, ConstU32<MAX_DOCUMENT_HASH_LENGTH>>,
    _marker: PhantomData<T>,
}

impl<T: Config> TryFrom<(T::AccountId, u64, Vec<u8>, Vec<u8>)> for TermsAndConditions<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided tuple
    /// (account id, timestamp, document link and document hash)
    /// No check for account id and timestamp.
    /// For document link and document hash check if byte vector
    /// is below the minimum or exceeds the maximum allowed length.
    /// Also check if byte vector only contains allowed ASCII characters.
    fn try_from(value: (T::AccountId, u64, Vec<u8>, Vec<u8>)) -> Result<Self, Self::Error> {
        // 1. account id
        let account_id = value.0;

        // 2. timestamp
        let timestamp = value.1;

        // 3. document link
        ensure!(
            value.2.len() >= MIN_DOCUMENT_LINK_LENGTH.saturated_into(),
            Self::Error::DocumentLinkInputToShort
        );
        let document_link: BoundedVec<u8, ConstU32<MAX_DOCUMENT_LINK_LENGTH>> =
            BoundedVec::try_from(value.2.clone())
                .map_err(|_| Self::Error::DocumentLinkInputToLong)?;
        ensure!(
            validate_document_link_input(&document_link),
            Self::Error::InvalidDocumentLinkInput
        );

        // 4. document hash
        ensure!(
            value.3.len() >= MIN_DOCUMENT_HASH_LENGTH.saturated_into(),
            Self::Error::DocumentHashInputToShort
        );
        let document_hash: BoundedVec<u8, ConstU32<MAX_DOCUMENT_HASH_LENGTH>> =
            BoundedVec::try_from(value.3.clone())
                .map_err(|_| Self::Error::DocumentHashInputToLong)?;
        ensure!(
            validate_document_hash_input(&document_hash),
            Self::Error::InvalidDocumentHashInput
        );

        Ok(Self {
            account_id,
            timestamp,
            document_link,
            document_hash,
            _marker: PhantomData,
        })
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for TermsAndConditions<T> {
    fn eq(&self, other: &Self) -> bool {
        self.account_id == other.account_id
            && self.timestamp == other.timestamp
            && self.document_link == other.document_link
            && self.document_hash == other.document_hash
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for TermsAndConditions<T> {
    fn clone(&self) -> Self {
        Self {
            account_id: self.account_id.clone(),
            timestamp: self.timestamp.clone(),
            document_link: self.document_link.clone(),
            document_hash: self.document_hash.clone(),
            _marker: PhantomData,
        }
    }
}

fn validate_document_link_input(input: &[u8]) -> bool {
    // TODO: find better alternative
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_'))
}

fn validate_document_hash_input(input: &[u8]) -> bool {
    // TODO: find better alternative
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_'))
}
