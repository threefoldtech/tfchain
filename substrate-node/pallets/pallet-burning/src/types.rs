use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Burn<AccountId, BalanceOf, BlockNumber> {
    pub target: AccountId,
    pub amount: BalanceOf,
    pub block: BlockNumber,
    pub message: Vec<u8>,
}
