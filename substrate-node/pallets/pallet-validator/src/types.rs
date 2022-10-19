use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, Debug, PartialEq, TypeInfo)]
pub struct Validator<AccountId> {
    pub validator_node_account: AccountId,
    pub stash_account: AccountId,
    pub description: Vec<u8>,
    pub tf_connect_id: Vec<u8>,
    pub info: Vec<u8>,
    pub state: ValidatorRequestState,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, TypeInfo)]
pub enum ValidatorRequestState {
    Created,
    Approved,
    Validating,
}
