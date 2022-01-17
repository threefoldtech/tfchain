use codec::{Decode, Encode};
use frame_support::traits::Vec;

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct ValidatorRequest<AccountId> {
    pub id: u32,
    pub validator_account: AccountId,
    pub stash_account: AccountId,
    pub description: Vec<u8>,
    pub tf_connect_id: u64,
    pub info: Vec<u8>,
}