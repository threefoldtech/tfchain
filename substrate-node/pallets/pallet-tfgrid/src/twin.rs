use sp_std::{marker::PhantomData, vec::Vec};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, sp_runtime::SaturatedConversion, BoundedVec, RuntimeDebug, traits::ConstU32};
use scale_info::TypeInfo;

use crate::ipv6;
use crate::{Config, Error};

/// A Twin planetary IP (ipv6).
///
/// It is bounded in size (inclusive range [MinLength, MaxLength]) and must be a valid ipv6
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct TwinIp<T: Config>(
    pub BoundedVec<u8, ConstU32<39>>,
    PhantomData<(T, ConstU32<39>)>,
);

pub const MIN_IP_LENGHT: u32 = 3;

impl<T: Config> TryFrom<Vec<u8>> for TwinIp<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP_LENGHT.saturated_into(),
            Self::Error::TwinIpTooShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<39>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::TwinIpTooLong)?;
        ensure!(ipv6::valid_ipv6(&bounded_vec), Self::Error::InvalidTwinIp);
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for TwinIp<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for TwinIp<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}
