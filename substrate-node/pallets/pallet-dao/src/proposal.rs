use codec::{Decode, Encode};
use frame_support::traits::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct DaoProposal<ProposalIndex> {
    pub index: ProposalIndex,
    pub description: Vec<u8>,
    pub link: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct Votes<ProposalIndex, AccountId, BlockNumber> {
    pub index: ProposalIndex,
    pub threshold: u32,
    pub ayes: Vec<VoteWeight<AccountId>>,
    pub nays: Vec<VoteWeight<AccountId>>,
    pub end: BlockNumber
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct VoteWeight<AccountId> {
    pub who: AccountId,
    pub weight: u64,
}