use codec::{Decode, Encode};
use frame_support::traits::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct DaoProposal<ProposalIndex> {
    pub index: ProposalIndex,
    pub description: Vec<u8>,
    pub link: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct Votes<ProposalIndex, BlockNumber> {
    pub index: ProposalIndex,
    pub threshold: u32,
    pub ayes: Vec<VoteWeight>,
    pub nays: Vec<VoteWeight>,
    pub end: BlockNumber
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct VoteWeight {
    pub farm_id: u32,
    pub weight: u64,
}