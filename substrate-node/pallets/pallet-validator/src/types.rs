use codec::{Decode, Encode};
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct Validator<AccountId> {
    pub validator_node_account: AccountId,
    pub stash_account: AccountId,
    pub description: Vec<u8>,
    pub tf_connect_id: Vec<u8>,
    pub info: Vec<u8>,
    pub state: ValidatorRequestState
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum ValidatorRequestState {
    Created,
    Approved,
    Validating
}