use codec::{Decode, Encode};
use frame_support::traits::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct Proposal<P> {
    pub action: Option<P>,
    pub description: Option<Vec<u8>>,
    pub link: Option<Vec<u8>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct Votes<ProposalIndex, AccountId, BlockNumber> {
    pub index: ProposalIndex,
    pub treshold: u32,
    pub ayes: Vec<AccountId>,
    pub nays: Vec<AccountId>,
    pub end: BlockNumber
}