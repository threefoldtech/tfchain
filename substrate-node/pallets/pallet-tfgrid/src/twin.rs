use crate::{Config, Error, TwinIpInput};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use valip::ip6::Ip;

/// A Twin planetary IP (ipv6).
///
/// It is bounded in size (inclusive range [MinLength, MaxLength]) and must be a valid ipv6
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct TwinIp<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_IP_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_IP_LENGTH>)>,
);

pub const MIN_IP_LENGTH: u32 = 3;
pub const MAX_IP_LENGTH: u32 = 39;

impl<T: Config> TryFrom<TwinIpInput> for TwinIp<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: TwinIpInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP_LENGTH.saturated_into(),
            Self::Error::TwinIpTooShort
        );
        ensure!(
            value.len() <= MAX_IP_LENGTH.saturated_into(),
            Self::Error::TwinIpTooLong
        );
        ensure!(Ip::parse(&value).is_ok(), Self::Error::InvalidTwinIp);
        Ok(Self(value, PhantomData))
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
