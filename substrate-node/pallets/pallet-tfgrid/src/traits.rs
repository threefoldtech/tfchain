use codec::{Codec, FullCodec};
use sp_std::fmt::Debug;
use sp_runtime::traits::MaybeSerializeDeserialize;

pub trait Tfgrid<AccountId> {
    type Farm: FullCodec + Debug;
    type Twin: FullCodec + Debug;

    fn get_farm(farm_id: u32) -> Self::Farm;
    fn get_twin(twin_id: u32) -> Self::Twin;
}